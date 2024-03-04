use std::collections::{HashMap, HashSet};

use crate::{
    builtins::Builtin,
    callable::{Callable, CallableResult, CallableSuccess, TailCallContext},
    compound_procedure::{Body, CompoundProcedure, Signature},
    interpreter::{RuntimeError, RuntimeErrorType},
    procedure::Procedure,
    source_mapped::{SourceMappable, SourceMapped},
    special_form::SpecialFormContext,
    string_interner::InternedString,
    value::{SourceValue, Value},
};

pub fn get_builtins() -> super::Builtins {
    vec![
        Builtin::SpecialForm("let", _let),
        Builtin::SpecialForm("let*", let_star),
        Builtin::SpecialForm("letrec", letrec),
    ]
}

struct LetBinding {
    variable: SourceMapped<InternedString>,
    init: SourceValue,
}

fn parse_bindings(
    ctx: &mut SpecialFormContext,
    allow_duplicates: bool,
    index: usize,
) -> Result<Vec<LetBinding>, RuntimeError> {
    let Some(bindings) = ctx
        .operands
        .get(index)
        .map(|value| value.try_into_list())
        .flatten()
    else {
        return Err(RuntimeErrorType::MalformedSpecialForm.source_mapped(ctx.range));
    };

    let mut result = Vec::with_capacity(bindings.0.len());
    let mut variables: HashSet<InternedString> = HashSet::with_capacity(bindings.0.len());

    for binding in bindings.0.iter() {
        let Some(binding) = binding.try_into_list() else {
            return Err(RuntimeErrorType::MalformedBindingList.source_mapped(binding.1));
        };
        if binding.0.len() != 2 {
            return Err(RuntimeErrorType::MalformedBindingList.source_mapped(binding.1));
        }
        let variable = binding.0[0].expect_identifier()?;
        let init = binding.0[1].clone();
        if !allow_duplicates && !variables.insert(variable.clone()) {
            return Err(RuntimeErrorType::DuplicateVariableInBindings.source_mapped(binding.0[0].1));
        }

        result.push(LetBinding {
            variable: variable.source_mapped(binding.0[0].1),
            init,
        })
    }

    Ok(result)
}

fn eval_body(ctx: &mut SpecialFormContext) -> Result<CallableSuccess, RuntimeError> {
    let body = Body::try_new(&ctx.operands[1..], ctx.range)?;
    ctx.interpreter
        .eval_expressions_in_tail_context(body.as_ref())
}

fn named_let(mut ctx: SpecialFormContext, variable: &InternedString) -> CallableResult {
    let bindings = parse_bindings(&mut ctx, false, 1)?;
    let mut arg_names = Vec::with_capacity(bindings.len());
    let mut arg_values = Vec::with_capacity(bindings.len());
    for binding in bindings {
        arg_names.push(binding.variable);
        arg_values.push(binding.init);
    }
    let signature: Signature = arg_names.into();
    let body = Body::try_new(&ctx.operands[2..], ctx.range)?;

    // This is a bit convoluted: we want to create a new scope that contains the name
    // of the let, so it can call itself recursively.
    ctx.interpreter.environment.push_inherited(ctx.range);
    let scope = ctx.interpreter.environment.capture_lexical_scope();
    let mut compound_procedure =
        CompoundProcedure::create(ctx.interpreter.new_id(), signature, body, scope);
    compound_procedure.name = Some(variable.clone());
    let proc = Procedure::Compound(compound_procedure);
    ctx.interpreter.environment.define(
        variable.clone(),
        Value::Callable(Callable::Procedure(proc.clone())).source_mapped(ctx.range),
    );
    ctx.interpreter.environment.pop();

    Ok(CallableSuccess::TailCall(TailCallContext {
        bound_procedure: proc.eval_and_bind(
            &mut ctx.interpreter,
            ctx.range,
            arg_values.as_ref(),
        )?,
    }))
}

fn _let(mut ctx: SpecialFormContext) -> CallableResult {
    if let Some(SourceMapped(Value::Symbol(variable), _)) = ctx.operands.get(0) {
        return named_let(ctx, variable);
    }
    let bindings = parse_bindings(&mut ctx, false, 0)?;
    let mut binding_map = HashMap::new();
    for binding in bindings.into_iter() {
        let value = ctx.interpreter.eval_expression(&binding.init)?;
        binding_map.insert(binding.variable.0, value);
    }
    ctx.interpreter.environment.push_inherited(ctx.range);
    for (variable, value) in binding_map {
        ctx.interpreter.environment.define(variable, value);
    }

    let result = eval_body(&mut ctx)?;

    // Note that the environment won't have been popped if an error occured above--this is
    // so we can examine it afterwards, if needed. It's up to the caller to clean things
    // up after an error.
    ctx.interpreter.environment.pop();

    Ok(result)
}

fn let_star(mut ctx: SpecialFormContext) -> CallableResult {
    let bindings = parse_bindings(&mut ctx, true, 0)?;
    let num_bindings = bindings.len();
    for binding in bindings.into_iter() {
        let value = ctx.interpreter.eval_expression(&binding.init)?;
        ctx.interpreter.environment.push_inherited(binding.init.1);
        ctx.interpreter
            .environment
            .define(binding.variable.0, value);
    }

    let result = eval_body(&mut ctx)?;

    // Note that the environment won't have been popped if an error occured above--this is
    // so we can examine it afterwards, if needed. It's up to the caller to clean things
    // up after an error.
    for _ in 0..num_bindings {
        ctx.interpreter.environment.pop();
    }

    Ok(result)
}

fn letrec(mut ctx: SpecialFormContext) -> CallableResult {
    let bindings = parse_bindings(&mut ctx, false, 0)?;
    ctx.interpreter.environment.push_inherited(ctx.range);
    for binding in bindings.into_iter() {
        let value = ctx.interpreter.eval_expression(&binding.init)?;
        ctx.interpreter
            .environment
            .define(binding.variable.0, value);
    }

    let result = eval_body(&mut ctx)?;

    // Note that the environment won't have been popped if an error occured above--this is
    // so we can examine it afterwards, if needed. It's up to the caller to clean things
    // up after an error.
    ctx.interpreter.environment.pop();

    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::{
        interpreter::RuntimeErrorType,
        test_util::{test_eval_err, test_eval_success},
    };

    #[test]
    fn let_works() {
        test_eval_success("(let ((x 1)) x)", "1");

        // From R5RS section 4.2.2.
        test_eval_success(
            "
            (let ((x 2) (y 3))
              (* x y))
            ",
            "6",
        );

        // From R5RS section 4.2.2.
        test_eval_success(
            "
            (let ((x 2) (y 3))
              (let ((x 7)
                    (z (+ x y)))
                (* z x)))
            ",
            "35",
        );
    }

    #[test]
    fn let_errors_on_bad_syntax() {
        test_eval_err("(let)", RuntimeErrorType::MalformedSpecialForm);
        test_eval_err("(let (x 1) x)", RuntimeErrorType::MalformedBindingList);
        test_eval_err("(let ((x 1 2)) x)", RuntimeErrorType::MalformedBindingList);
        test_eval_err("(let ((x 1)))", RuntimeErrorType::MalformedSpecialForm);
        test_eval_err("(let ((1 1)) x)", RuntimeErrorType::ExpectedIdentifier);
        test_eval_err(
            "(let ((x 1) (x 2)) x)",
            RuntimeErrorType::DuplicateVariableInBindings,
        );
    }

    #[test]
    fn let_star_works() {
        // Note that duplicates are OK!
        test_eval_success("(let* ((x 1) (x 2)) x)", "2");

        // From R5RS section 4.2.2.
        test_eval_success(
            "
            (let ((x 2) (y 3))
              (let* ((x 7)
                     (z (+ x y)))
                (* z x)))
            ",
            "70",
        );
    }

    #[test]
    fn let_star_errors_on_bad_syntax() {
        test_eval_err("(let*)", RuntimeErrorType::MalformedSpecialForm);
        test_eval_err("(let* (x 1) x)", RuntimeErrorType::MalformedBindingList);
        test_eval_err("(let* ((x 1 2)) x)", RuntimeErrorType::MalformedBindingList);
        test_eval_err("(let* ((x 1)))", RuntimeErrorType::MalformedSpecialForm);
        test_eval_err("(let* ((1 1)) x)", RuntimeErrorType::ExpectedIdentifier);
    }

    #[test]
    fn letrec_works() {
        // From R5RS section 4.2.2.
        test_eval_success(
            "
        (letrec ((even?
                  (lambda (n)
                    (if (zero? n)
                        #t
                        (odd? (- n 1)))))
                 (odd?
                  (lambda (n)
                    (if (zero? n)
                        #f
                        (even? (- n 1))))))
          (even? 88))
        ",
            "#t",
        )
    }

    #[test]
    fn letrec_errors_on_bad_syntax() {
        test_eval_err("(letrec)", RuntimeErrorType::MalformedSpecialForm);
        test_eval_err("(letrec (x 1) x)", RuntimeErrorType::MalformedBindingList);
        test_eval_err(
            "(letrec ((x 1 2)) x)",
            RuntimeErrorType::MalformedBindingList,
        );
        test_eval_err("(letrec ((x 1)))", RuntimeErrorType::MalformedSpecialForm);
        test_eval_err("(letrec ((1 1)) x)", RuntimeErrorType::ExpectedIdentifier);
        test_eval_err(
            "(letrec ((x 1) (x 2)) x)",
            RuntimeErrorType::DuplicateVariableInBindings,
        );
    }

    #[test]
    fn named_let_works() {
        test_eval_success(
            "
            (let blap
              ((x 100) (y 0))
              (if (= x 1)
                y
                (blap (- x 1) (+ 1 y))
              )
            )
            ",
            "99",
        );
    }

    #[test]
    fn named_let_errors_on_bad_syntax() {
        test_eval_err("(let boop)", RuntimeErrorType::MalformedSpecialForm);
        test_eval_err("(let boop (x 1) x)", RuntimeErrorType::MalformedBindingList);
        test_eval_err(
            "(let boop ((x 1 2)) x)",
            RuntimeErrorType::MalformedBindingList,
        );
        test_eval_err("(let boop ((x 1)))", RuntimeErrorType::MalformedSpecialForm);
        test_eval_err("(let boop ((1 1)) x)", RuntimeErrorType::ExpectedIdentifier);
        test_eval_err(
            "(let boop ((x 1) (x 2)) x)",
            RuntimeErrorType::DuplicateVariableInBindings,
        );
    }
}
