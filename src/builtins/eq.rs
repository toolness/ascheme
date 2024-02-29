use crate::{
    builtin_procedure::{BuiltinProcedureContext, BuiltinProcedureFn},
    builtins::Builtin,
    interpreter::{Callable, CallableResult, RuntimeError},
    procedure::Procedure,
    value::{SourceValue, Value},
};

pub fn get_builtins() -> super::Builtins {
    vec![Builtin::Procedure("eq?", BuiltinProcedureFn::Binary(eq))]
}

pub fn is_eq(a: &SourceValue, b: &SourceValue) -> Result<bool, RuntimeError> {
    Ok(match &a.0 {
        Value::Undefined => matches!(b.0, Value::Undefined),
        Value::EmptyList => matches!(b.0, Value::EmptyList),
        Value::Number(a) => match b.0 {
            Value::Number(b) => a == &b,
            _ => false,
        },
        Value::Symbol(a) => match &b.0 {
            Value::Symbol(b) => a == b,
            _ => false,
        },
        Value::Boolean(a) => match b.0 {
            Value::Boolean(b) => a == &b,
            _ => false,
        },
        Value::String(a) => match &b.0 {
            Value::String(b) => a.points_at_same_memory_as(b),
            _ => false,
        },
        Value::Callable(Callable::SpecialForm(a)) => match &b.0 {
            Value::Callable(Callable::SpecialForm(b)) => a.func == b.func,
            _ => false,
        },
        Value::Callable(Callable::Procedure(Procedure::Builtin(a))) => match &b.0 {
            Value::Callable(Callable::Procedure(Procedure::Builtin(b))) => a.func == b.func,
            _ => false,
        },
        Value::Callable(Callable::Procedure(Procedure::Compound(a))) => match &b.0 {
            Value::Callable(Callable::Procedure(Procedure::Compound(b))) => a.id() == b.id(),
            _ => false,
        },
        Value::Pair(a) => match &b.0 {
            Value::Pair(b) => a.points_at_same_memory_as(b),
            _ => false,
        },
    })
}

fn eq(_ctx: BuiltinProcedureContext, a: &SourceValue, b: &SourceValue) -> CallableResult {
    Ok(is_eq(a, b)?.into())
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
