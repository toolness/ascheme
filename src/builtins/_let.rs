use std::collections::{HashMap, HashSet};

use crate::{
    interpreter::{
        ProcedureContext, ProcedureResult, ProcedureSuccess, RuntimeError, RuntimeErrorType,
    },
    source_mapped::SourceMappable,
    string_interner::InternedString,
    value::SourceValue,
};

pub fn get_builtins() -> super::Builtins {
    vec![("let", _let), ("letrec", letrec)]
}

struct LetBinding {
    variable: InternedString,
    init: SourceValue,
}

fn parse_bindings(ctx: &mut ProcedureContext) -> Result<Vec<LetBinding>, RuntimeError> {
    let Some(bindings) = ctx
        .operands
        .get(0)
        .map(|value| value.try_into_list())
        .flatten()
    else {
        return Err(RuntimeErrorType::MalformedSpecialForm.source_mapped(ctx.combination.1));
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
        if !variables.insert(variable.clone()) {
            return Err(RuntimeErrorType::DuplicateVariableInBindings.source_mapped(binding.0[0].1));
        }

        result.push(LetBinding { variable, init })
    }

    Ok(result)
}

fn eval_body(ctx: &mut ProcedureContext) -> Result<ProcedureSuccess, RuntimeError> {
    let body = &ctx.operands[1..];
    if body.is_empty() {
        return Err(RuntimeErrorType::MalformedSpecialForm.source_mapped(ctx.combination.1));
    }
    ctx.interpreter.eval_expressions_in_tail_context(body)
}

fn _let(mut ctx: ProcedureContext) -> ProcedureResult {
    let bindings = parse_bindings(&mut ctx)?;
    let mut binding_map = HashMap::new();
    for binding in bindings.into_iter() {
        let value = ctx.interpreter.eval_expression(&binding.init)?;
        binding_map.insert(binding.variable, value);
    }
    let scope = ctx.interpreter.environment.capture_lexical_scope();
    ctx.interpreter.environment.push(scope, ctx.combination.1);
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

fn letrec(mut ctx: ProcedureContext) -> ProcedureResult {
    let bindings = parse_bindings(&mut ctx)?;
    let scope = ctx.interpreter.environment.capture_lexical_scope();
    ctx.interpreter.environment.push(scope, ctx.combination.1);
    for binding in bindings.into_iter() {
        let value = ctx.interpreter.eval_expression(&binding.init)?;
        ctx.interpreter.environment.define(binding.variable, value);
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
}
