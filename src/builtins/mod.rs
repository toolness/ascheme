use crate::{
    compound_procedure::{Body, CompoundProcedure, Signature},
    environment::Environment,
    interpreter::{Procedure, ProcedureContext, ProcedureFn, ProcedureResult, RuntimeErrorType},
    source_mapped::{SourceMappable, SourceMapped},
    string_interner::StringInterner,
    value::Value,
};

mod _let;
mod eq;
mod library;
mod logic;
mod math;
mod non_standard;
mod ord;
mod pair;
mod util;

pub use library::add_library_source;

pub fn populate_environment(environment: &mut Environment, interner: &mut StringInterner) {
    for (name, builtin) in get_builtins() {
        let interned_name = interner.intern(name);
        environment.define(
            interned_name.clone(),
            Value::Procedure(Procedure::Builtin(builtin, interned_name)).into(),
        );
    }
    // TODO: Technically 'else' is just part of how the 'cond' special form is evaluated,
    // but just aliasing it to 'true' is easier for now.
    environment.define(interner.intern("else"), Value::Boolean(true).into());
}

pub type Builtins = Vec<(&'static str, ProcedureFn)>;

fn get_builtins() -> Builtins {
    let mut builtins: Builtins = vec![
        ("define", define),
        ("lambda", lambda),
        ("apply", apply),
        ("quote", quote),
        ("begin", begin),
        ("display", display),
        ("if", _if),
        ("cond", cond),
        ("set!", set),
    ];
    builtins.extend(math::get_builtins());
    builtins.extend(eq::get_builtins());
    builtins.extend(ord::get_builtins());
    builtins.extend(logic::get_builtins());
    builtins.extend(non_standard::get_builtins());
    builtins.extend(_let::get_builtins());
    builtins.extend(pair::get_builtins());
    builtins
}

fn _if(ctx: ProcedureContext) -> ProcedureResult {
    if ctx.operands.len() < 2 || ctx.operands.len() > 3 {
        return Err(RuntimeErrorType::MalformedSpecialForm.source_mapped(ctx.range));
    }
    let test = ctx.interpreter.eval_expression(&ctx.operands[0])?.0;
    if test.as_bool() {
        let consequent_expr = &ctx.operands[1];
        ctx.interpreter
            .eval_expression_in_tail_context(consequent_expr)
    } else {
        if let Some(alternate_expr) = ctx.operands.get(2) {
            ctx.interpreter
                .eval_expression_in_tail_context(alternate_expr)
        } else {
            ctx.undefined()
        }
    }
}

fn cond(ctx: ProcedureContext) -> ProcedureResult {
    if ctx.operands.len() == 0 {
        return Err(RuntimeErrorType::MalformedSpecialForm.source_mapped(ctx.range));
    }

    for clause in ctx.operands.iter() {
        let SourceMapped(Value::Pair(pair), range) = clause else {
            return Err(RuntimeErrorType::MalformedSpecialForm.source_mapped(clause.1));
        };
        let Some(clause) = pair.try_as_rc_list() else {
            return Err(RuntimeErrorType::MalformedSpecialForm.source_mapped(*range));
        };
        let test = ctx.interpreter.eval_expression(&clause[0])?.0;
        if test.as_bool() {
            if clause.len() == 1 {
                return Ok(test.into());
            }
            return ctx
                .interpreter
                .eval_expressions_in_tail_context(&clause[1..]);
        }
    }

    ctx.undefined()
}

// TODO: According to R5RS section 5.2, definitions are only allowed at the top level
// of a program file, and at the beginning of a body. Currently we support it anywhere.
fn define(ctx: ProcedureContext) -> ProcedureResult {
    match ctx.operands.get(0) {
        Some(SourceMapped(Value::Symbol(name), ..)) => {
            let mut value = ctx.interpreter.eval_expressions(&ctx.operands[1..])?;
            if let Value::Procedure(Procedure::Compound(compound)) = &mut value.0 {
                if compound.name.is_none() {
                    compound.name = Some(name.clone());
                }
            }
            ctx.interpreter.environment.define(name.clone(), value);
            ctx.undefined()
        }
        Some(SourceMapped(Value::Pair(pair), range)) => {
            let name = pair.car().expect_identifier()?;
            let signature = Signature::parse(pair.cdr())?;
            let body = Body::try_new(&ctx.operands[1..], ctx.range)?;
            let mut proc = CompoundProcedure::create(
                ctx.interpreter.new_id(),
                signature,
                body,
                ctx.interpreter.environment.capture_lexical_scope(),
            );
            proc.name = Some(name.clone());
            ctx.interpreter.environment.define(
                name,
                Value::Procedure(Procedure::Compound(proc)).source_mapped(*range),
            );
            ctx.undefined()
        }
        _ => Err(RuntimeErrorType::MalformedSpecialForm.source_mapped(ctx.range)),
    }
}

fn lambda(ctx: ProcedureContext) -> ProcedureResult {
    if ctx.operands.len() < 2 {
        return Err(RuntimeErrorType::MalformedSpecialForm.source_mapped(ctx.range));
    }
    let signature = Signature::parse(ctx.operands[0].clone())?;
    let body = Body::try_new(&ctx.operands[1..], ctx.range)?;
    let proc = CompoundProcedure::create(
        ctx.interpreter.new_id(),
        signature,
        body,
        ctx.interpreter.environment.capture_lexical_scope(),
    );
    Ok(Value::Procedure(Procedure::Compound(proc)).into())
}

fn apply(ctx: ProcedureContext) -> ProcedureResult {
    ctx.ensure_operands_len(2)?;
    let procedure = ctx.interpreter.expect_procedure(&ctx.operands[0])?;
    let operands = ctx
        .interpreter
        .eval_expression(&ctx.operands[1])?
        .expect_list()?;
    ctx.interpreter
        .eval_procedure(procedure, &operands, ctx.operands[0].1, ctx.range)
}

fn quote(ctx: ProcedureContext) -> ProcedureResult {
    if ctx.operands.len() == 1 {
        Ok(ctx.operands[0].clone().into())
    } else {
        Err(RuntimeErrorType::MalformedSpecialForm.source_mapped(ctx.range))
    }
}

fn begin(ctx: ProcedureContext) -> ProcedureResult {
    ctx.interpreter
        .eval_expressions_in_tail_context(&ctx.operands)
}

fn set(ctx: ProcedureContext) -> ProcedureResult {
    ctx.ensure_operands_len(2)?;
    let identifier = ctx.operands[0].expect_identifier()?;
    let value = ctx.interpreter.eval_expression(&ctx.operands[1])?;
    if let Err(err) = ctx.interpreter.environment.change(&identifier, value) {
        Err(err.source_mapped(ctx.operands[0].1))
    } else {
        ctx.undefined()
    }
}

fn display(mut ctx: ProcedureContext) -> ProcedureResult {
    let value = ctx.eval_unary()?;
    ctx.interpreter.printer.print(format!("{:#}", value));
    ctx.undefined()
}

#[cfg(test)]
mod tests {
    use crate::{
        interpreter::RuntimeErrorType,
        test_util::{test_eval_err, test_eval_success, test_eval_successes},
    };

    #[test]
    fn quote_works() {
        test_eval_success("(quote 1)", "1");
        test_eval_success("(quote (1   2    3  ))", "(1 2 3)");
        test_eval_success("(quote (1 2 3 (4)))", "(1 2 3 (4))");
        test_eval_success("(quote #t)", "#t");
        test_eval_success("(quote #f)", "#f");
        test_eval_success("(quote ())", "()");
        test_eval_success("(quote blarg)", "blarg");

        test_eval_success("'1", "1");
        test_eval_success("'(1   2    3  )", "(1 2 3)");
        test_eval_success("'(1 2 3 (4))", "(1 2 3 (4))");
        test_eval_success("'#t", "#t");
        test_eval_success("'#f", "#f");
        test_eval_success("'()", "()");
        test_eval_success("'blarg", "blarg");
    }

    #[test]
    fn cond_works() {
        test_eval_success("(cond (1))", "1");
        test_eval_success("(cond (0))", "0");
        test_eval_success("(cond (#f))", "");
        test_eval_success("(cond (1 2 3 4))", "4");
        test_eval_success("(cond (#f 1) (else (+ 1 1)))", "2");
        test_eval_success("(cond (1) (lolol))", "1");
    }

    #[test]
    fn variable_definitions_work() {
        test_eval_success("(define x 3) x", "3");
        test_eval_success("(define x 3) (define y (+ x 1)) (+ x y)", "7");
        test_eval_success("(define x) x", "");
    }

    #[test]
    fn compound_procedure_definitions_with_fixed_args_work() {
        test_eval_success("(define (x) 3)", "");
        test_eval_success("(define (x) 3) (x)", "3");
        test_eval_success("(define (add-three n) (+ 3 n)) (add-three 1)", "4");
        test_eval_success(
            "(define x 9) (define (boop x y) (+ x y)) (boop 1 (+ x 1))",
            "11",
        );
    }

    #[test]
    fn compound_procedure_definitions_with_any_args_work() {
        test_eval_success("(define (n . z) z)", "");
        test_eval_success("(define (n . z) z) (n)", "()");
        test_eval_success("(define (n . z) z) (n 1 2)", "(1 2)");
    }

    #[test]
    fn compound_procedure_definitions_with_min_args_work() {
        test_eval_success("(define (n a . z) z) (n 1 2)", "(2)");
        test_eval_success("(define (n a . z) z) (n 1)", "()");
    }

    #[test]
    fn procedures_raise_wrong_number_of_args_errors() {
        test_eval_err("((lambda (x) x))", RuntimeErrorType::WrongNumberOfArguments);
        test_eval_err(
            "((lambda (x y . z) 1) 1)",
            RuntimeErrorType::WrongNumberOfArguments,
        );
        test_eval_err(
            "(define (x a) a) (x)",
            RuntimeErrorType::WrongNumberOfArguments,
        );
        test_eval_err(
            "(define (x a b . c) a) (x 1)",
            RuntimeErrorType::WrongNumberOfArguments,
        );
    }

    #[test]
    fn define_errors_on_duplicate_parameters() {
        test_eval_err("(define (foo x x) 3)", RuntimeErrorType::DuplicateParameter);
    }

    #[test]
    fn define_errors_on_no_body() {
        test_eval_err("(define (a))", RuntimeErrorType::MalformedSpecialForm);
    }

    #[test]
    fn lambda_fixed_arg_definitions_work() {
        test_eval_success("(define x (lambda () 3))", "");
        test_eval_success("(define x (lambda () 3)) (x)", "3");
        test_eval_success("(define add-three (lambda (n) (+ 3 n))) (add-three 1)", "4");
        test_eval_success(
            "(define x 9) (define boop (lambda (x y) (+ x y))) (boop 1 (+ x 1))",
            "11",
        );
    }

    #[test]
    fn lambda_any_arg_definitions_work() {
        test_eval_success("(define x (lambda n n))", "");
        test_eval_success("(define x (lambda n n)) (x)", "()");
        test_eval_success("(define x (lambda n n)) (x 1 2 3)", "(1 2 3)");
    }

    #[test]
    fn lambda_min_arg_definitions_work() {
        test_eval_success("(define x (lambda (n . z) n))", "");
        test_eval_success("(define x (lambda (n . z) n)) (x 5 4 3 2 1)", "5");
        test_eval_success("(define x (lambda (n . z) z)) (x 5 4 3 2 1)", "(4 3 2 1)");
    }

    #[test]
    fn lambda_errors_on_duplicate_parameters() {
        test_eval_err("(lambda (a a) 3)", RuntimeErrorType::DuplicateParameter);
    }

    #[test]
    fn lambda_errors_on_no_body() {
        test_eval_err("(lambda (a))", RuntimeErrorType::MalformedSpecialForm);
    }

    #[test]
    fn set_works_with_globals() {
        test_eval_success("(define x 1) (set! x 2) x", "2");
        test_eval_success("(define x 1) (set! x (+ x 1)) x", "2");
    }

    #[test]
    fn set_works_in_closures() {
        test_eval_successes(&[
            (
                "
                (define (make-incrementer)
                  (define n 0)
                  (lambda ()
                    (set! n (+ n 1))
                    n
                  )
                )
                (define foo (make-incrementer)) 
                (define bar (make-incrementer)) 
                ",
                "",
            ),
            ("(foo)", "1"),
            ("(foo)", "2"),
            ("(foo)", "3"),
            ("(bar)", "1"),
            ("(bar)", "2"),
        ]);
    }

    #[test]
    fn if_works() {
        test_eval_success("(if #t 1)", "1");
        test_eval_success("(if #t 1 2)", "1");
        test_eval_success("(if #f 1 2)", "2");

        // R5RS section 4.1.5 says this behavior is unspecified, we'll just return undefined.
        test_eval_success("(if #f 1)", "");
    }

    #[test]
    fn compound_procedues_prefer_argument_values_to_globals() {
        test_eval_success(
            "
            (define n 5)
            (define (add-three n) (+ 3 n))
            (+ (add-three 1) n)
        ",
            "9",
        );
    }

    #[test]
    fn compound_procedues_use_lexical_scope() {
        test_eval_success(
            "
            (define (make-adder n)
              (define (add-n x) (+ x n))
              add-n
            )
            (define add-three (make-adder 3))
            (add-three 1)
        ",
            "4",
        );
    }

    #[test]
    fn display_works() {
        test_eval_success(r#"(display "boop")"#, "boop");
        test_eval_success(r#"(display '("boop"))"#, "(boop)");
        test_eval_success(r#"(display 1)"#, "1");
    }

    #[test]
    fn begin_works() {
        test_eval_success("(begin)", "");
        test_eval_success("(begin 1)", "1");
        test_eval_success("(begin 1 2)", "2");
        test_eval_success("(begin (+ 1 2))", "3");
    }

    #[test]
    fn apply_works() {
        // From R5RS 6.4.
        test_eval_success("(apply + (list 3 4))", "7");

        // From R5RS 6.4.
        test_eval_success(
            "
            (define compose
                (lambda (f g)
                  (lambda args
                    (f (apply g args)))))
            ((compose sqrt *) 12 75)
            ",
            "30",
        );

        test_eval_success("(apply + '())", "0");
    }
}
