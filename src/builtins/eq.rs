use crate::{
    interpreter::{
        Interpreter, Procedure, ProcedureContext, ProcedureResult, RuntimeError, RuntimeErrorType,
    },
    source_mapped::SourceMappable,
    value::{SourceValue, Value},
};

pub fn is_eq(
    interpreter: &mut Interpreter,
    a: &SourceValue,
    b: &SourceValue,
) -> Result<bool, RuntimeError> {
    let a = interpreter.eval_expression(&a)?;
    let b = interpreter.eval_expression(&b)?;

    Ok(match a.0 {
        Value::Undefined => matches!(b.0, Value::Undefined),
        Value::EmptyList => matches!(b.0, Value::EmptyList),
        Value::Number(a) => match b.0 {
            Value::Number(b) => a == b,
            _ => false,
        },
        Value::Symbol(a) => match &b.0 {
            Value::Symbol(b) => &a == b,
            _ => false,
        },
        Value::Boolean(a) => match b.0 {
            Value::Boolean(b) => a == b,
            _ => false,
        },
        Value::String(a) => match &b.0 {
            Value::String(b) => a.points_at_same_memory_as(b),
            _ => false,
        },
        Value::Procedure(Procedure::Builtin(a, _)) => match &b.0 {
            Value::Procedure(Procedure::Builtin(b, _)) => a == *b,
            _ => false,
        },
        Value::Procedure(Procedure::Compound(a)) => match &b.0 {
            Value::Procedure(Procedure::Compound(b)) => a.id() == b.id(),
            _ => false,
        },
        Value::Pair(a) => match &b.0 {
            Value::Pair(b) => a.points_at_same_memory_as(b),
            _ => false,
        },
    })
}

pub fn eq(mut ctx: ProcedureContext) -> ProcedureResult {
    if ctx.operands.len() < 2 {
        return Err(RuntimeErrorType::WrongNumberOfArguments.source_mapped(ctx.combination.1));
    }

    Ok(is_eq(&mut ctx.interpreter, &ctx.operands[0], &ctx.operands[1])?.into())
}

#[cfg(test)]
mod tests {
    use crate::test_util::test_eval_success;

    #[test]
    fn eq_works() {
        // From R5RS section 6.1.
        test_eval_success("(eq? (quote a) (quote a))", "#t");
        test_eval_success("(eq? (quote a) (quote b))", "#f");
        test_eval_success("(eq? (quote ()) (quote ()))", "#t");
        test_eval_success("(eq? + +)", "#t");
        test_eval_success("(eq? 2 2)", "#t");
        test_eval_success("(eq? 2 1)", "#f");
        test_eval_success("(define (x) (x)) (eq? x x)", "#t");
        test_eval_success("(eq? #t #f)", "#f");
        test_eval_success("(eq? #t #t)", "#t");
        test_eval_success("(eq? #f #f)", "#t");

        // Stuff specific to our implementation.
        test_eval_success("(eq? (quote (a)) (quote (a)))", "#f");
        test_eval_success("(eq? (quote (1 . 2)) (quote (1 . 2)))", "#f");
        test_eval_success("(define x (quote (a))) (eq? x x)", "#t");
        test_eval_success("(eq? (lambda (x) (x)) (lambda (x) (x)))", "#f");
    }

    #[test]
    fn strings_work() {
        test_eval_success(r#"(eq? "blarg" "blarg")"#, "#f");
        test_eval_success(r#"(define x "blarg") (eq? x x)"#, "#t");
    }
}
