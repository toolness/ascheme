use crate::{
    builtin_procedure::{BuiltinProcedureContext, BuiltinProcedureFn},
    builtins::Builtin,
    callable::CallableResult,
    source_mapped::SourceMappable,
    value::{SourceValue, Value},
};

use super::Builtins;

pub fn get_builtins() -> Builtins {
    vec![
        Builtin::Procedure("set-car!", BuiltinProcedureFn::Binary(set_car)),
        Builtin::Procedure("set-cdr!", BuiltinProcedureFn::Binary(set_cdr)),
        Builtin::Procedure("cons", BuiltinProcedureFn::Binary(cons)),
        Builtin::Procedure("car", BuiltinProcedureFn::Unary(car)),
        Builtin::Procedure("cdr", BuiltinProcedureFn::Unary(cdr)),
        Builtin::Procedure("list", BuiltinProcedureFn::NullaryVariadic(list)),
        Builtin::Procedure("pair?", BuiltinProcedureFn::Unary(pair)),
    ]
}

fn set_car(
    ctx: BuiltinProcedureContext,
    pair: &SourceValue,
    value: &SourceValue,
) -> CallableResult {
    let mut pair = pair.expect_pair()?;
    pair.set_car(value.clone());
    ctx.undefined()
}

fn set_cdr(
    ctx: BuiltinProcedureContext,
    pair: &SourceValue,
    value: &SourceValue,
) -> CallableResult {
    let mut pair = pair.expect_pair()?;
    pair.set_cdr(value.clone());
    ctx.undefined()
}

fn car(_ctx: BuiltinProcedureContext, value: &SourceValue) -> CallableResult {
    Ok(value.expect_pair()?.car().into())
}

fn cdr(_ctx: BuiltinProcedureContext, value: &SourceValue) -> CallableResult {
    Ok(value.expect_pair()?.cdr().into())
}

fn cons(ctx: BuiltinProcedureContext, car: &SourceValue, cdr: &SourceValue) -> CallableResult {
    let pair = Value::Pair(ctx.interpreter.pair_manager.pair(car.clone(), cdr.clone()))
        .source_mapped(ctx.range);
    Ok(pair.into())
}

fn list(ctx: BuiltinProcedureContext, operands: &[SourceValue]) -> CallableResult {
    Ok(ctx
        .interpreter
        .pair_manager
        .vec_to_list(operands.into())
        .into())
}

fn pair(_ctx: BuiltinProcedureContext, operand: &SourceValue) -> CallableResult {
    Ok(matches!(operand.0, Value::Pair(_)).into())
}

#[cfg(test)]
mod tests {
    use crate::test_util::test_eval_success;

    #[test]
    fn set_car_works() {
        test_eval_success("(define a (quote (1 . 2))) (set-car! a 5) a", "(5 . 2)");
    }

    #[test]
    fn set_cdr_works() {
        test_eval_success("(define a (quote (1 . 2))) (set-cdr! a 5) a", "(1 . 5)");
    }

    #[test]
    fn cons_works() {
        test_eval_success("(cons 1 2)", "(1 . 2)");
        test_eval_success("(cons 1 '())", "(1)");
    }

    #[test]
    fn car_works() {
        test_eval_success("(car '(1 . 2))", "1");
    }

    #[test]
    fn cdr_works() {
        test_eval_success("(cdr '(1 . 2))", "2");
    }

    #[test]
    fn list_works() {
        test_eval_success("(list)", "()");
        test_eval_success("(list 1 2 3)", "(1 2 3)");
        test_eval_success("(list (+ 1 2))", "(3)");
    }

    #[test]
    fn pair_works() {
        test_eval_success("(pair? 1)", "#f");
        test_eval_success("(pair? '())", "#f");
        test_eval_success("(pair? '(1 . 2))", "#t");
        test_eval_success("(pair? '(1 2))", "#t");
    }
}
