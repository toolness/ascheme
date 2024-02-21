use crate::{
    compound_procedure::CompoundProcedure,
    environment::Environment,
    interpreter::{
        Procedure, ProcedureContext, ProcedureFn, ProcedureResult, RuntimeError, RuntimeErrorType,
    },
    pair::Pair,
    source_mapped::{SourceMappable, SourceMapped},
    string_interner::StringInterner,
    value::{SourceValue, Value},
};

mod eq;
mod library;
mod logic;
mod math;
mod non_standard;
mod ord;
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
        ("quote", quote),
        ("display", display),
        ("if", _if),
        ("cond", cond),
        ("set!", set),
        ("set-car!", set_car),
        ("set-cdr!", set_cdr),
    ];
    builtins.extend(math::get_builtins());
    builtins.extend(eq::get_builtins());
    builtins.extend(ord::get_builtins());
    builtins.extend(logic::get_builtins());
    builtins.extend(non_standard::get_builtins());
    builtins
}

fn _if(ctx: ProcedureContext) -> ProcedureResult {
    if ctx.operands.len() < 2 || ctx.operands.len() > 3 {
        return Err(RuntimeErrorType::MalformedSpecialForm.source_mapped(ctx.combination.1));
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
        return Err(RuntimeErrorType::MalformedSpecialForm.source_mapped(ctx.combination.1));
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
            let Some(expressions) = pair.try_as_rc_list() else {
                return Err(RuntimeErrorType::MalformedSpecialForm.source_mapped(*range));
            };
            let signature = SourceMapped(expressions, *range);
            // We can just unwrap this b/c it's from a pair.
            let first = signature.0.get(0).unwrap();
            let name = first.expect_identifier()?;
            let mut proc = CompoundProcedure::create(
                ctx.interpreter.new_id(),
                signature,
                1,
                SourceMapped(ctx.combination.0.clone(), ctx.combination.1),
                ctx.interpreter.environment.capture_lexical_scope(),
            )?;
            proc.name = Some(name.clone());
            ctx.interpreter.environment.define(
                name,
                Value::Procedure(Procedure::Compound(proc)).source_mapped(*range),
            );
            ctx.undefined()
        }
        _ => Err(RuntimeErrorType::MalformedSpecialForm.source_mapped(ctx.combination.1)),
    }
}

fn lambda(ctx: ProcedureContext) -> ProcedureResult {
    let Some(SourceMapped(expressions, range)) = ctx
        .operands
        .get(0)
        .map(|value| value.try_into_list())
        .flatten()
    else {
        return Err(RuntimeErrorType::MalformedSpecialForm.source_mapped(ctx.combination.1));
    };
    let signature = SourceMapped(expressions.clone(), range);
    let proc = CompoundProcedure::create(
        ctx.interpreter.new_id(),
        signature,
        0,
        SourceMapped(ctx.combination.0.clone(), ctx.combination.1),
        ctx.interpreter.environment.capture_lexical_scope(),
    )?;
    Ok(Value::Procedure(Procedure::Compound(proc)).into())
}

fn quote(ctx: ProcedureContext) -> ProcedureResult {
    if ctx.operands.len() == 1 {
        Ok(ctx.operands[0].clone().into())
    } else {
        Err(RuntimeErrorType::MalformedSpecialForm.source_mapped(ctx.combination.1))
    }
}

fn eval_pair_and_value(ctx: &mut ProcedureContext) -> Result<(Pair, SourceValue), RuntimeError> {
    ctx.ensure_operands_len(2)?;
    let pair = ctx
        .interpreter
        .eval_expression(&ctx.operands[0])?
        .expect_pair()?;
    let value = ctx.interpreter.eval_expression(&ctx.operands[1])?;
    Ok((pair, value))
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

fn set_car(mut ctx: ProcedureContext) -> ProcedureResult {
    let (mut pair, value) = eval_pair_and_value(&mut ctx)?;
    pair.set_car(value);
    ctx.undefined()
}

fn set_cdr(mut ctx: ProcedureContext) -> ProcedureResult {
    let (mut pair, value) = eval_pair_and_value(&mut ctx)?;
    pair.set_cdr(value);
    ctx.undefined()
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
    fn set_car_works() {
        test_eval_success("(define a (quote (1 . 2))) (set-car! a 5) a", "(5 . 2)");
    }

    #[test]
    fn set_cdr_works() {
        test_eval_success("(define a (quote (1 . 2))) (set-cdr! a 5) a", "(1 . 5)");
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
    }

    #[test]
    fn compound_procedure_definitions_work() {
        test_eval_success("(define (x) 3)", "");
        test_eval_success("(define (x) 3) (x)", "3");
        test_eval_success("(define (add-three n) (+ 3 n)) (add-three 1)", "4");
    }

    #[test]
    fn define_errors_on_duplicate_parameters() {
        test_eval_err("(define (foo x x) 3)", RuntimeErrorType::DuplicateParameter);
    }

    #[test]
    fn lambda_definitions_work() {
        test_eval_success("(define x (lambda () 3))", "");
        test_eval_success("(define x (lambda () 3)) (x)", "3");
        test_eval_success("(define add-three (lambda (n) (+ 3 n))) (add-three 1)", "4");
    }

    #[test]
    fn lambda_errors_on_duplicate_parameters() {
        test_eval_err("(lambda (a a) 3)", RuntimeErrorType::DuplicateParameter);
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
}
