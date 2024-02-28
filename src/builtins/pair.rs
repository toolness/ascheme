use crate::{
    builtins::Builtin,
    interpreter::{CallableResult, RuntimeError, SpecialFormContext},
    pair::Pair,
    source_mapped::SourceMappable,
    value::{SourceValue, Value},
};

use super::Builtins;

pub fn get_builtins() -> Builtins {
    vec![
        // TODO: I think these are all procedures, not special forms.
        Builtin::SpecialForm("set-car!", set_car),
        Builtin::SpecialForm("set-cdr!", set_cdr),
        Builtin::SpecialForm("cons", cons),
        Builtin::SpecialForm("car", car),
        Builtin::SpecialForm("cdr", cdr),
        Builtin::SpecialForm("list", list),
        Builtin::SpecialForm("pair?", pair),
    ]
}

fn eval_pair_and_value(ctx: &mut SpecialFormContext) -> Result<(Pair, SourceValue), RuntimeError> {
    ctx.ensure_operands_len(2)?;
    let pair = ctx
        .interpreter
        .eval_expression(&ctx.operands[0])?
        .expect_pair()?;
    let value = ctx.interpreter.eval_expression(&ctx.operands[1])?;
    Ok((pair, value))
}

fn set_car(mut ctx: SpecialFormContext) -> CallableResult {
    let (mut pair, value) = eval_pair_and_value(&mut ctx)?;
    pair.set_car(value);
    ctx.undefined()
}

fn set_cdr(mut ctx: SpecialFormContext) -> CallableResult {
    let (mut pair, value) = eval_pair_and_value(&mut ctx)?;
    pair.set_cdr(value);
    ctx.undefined()
}

fn car(mut ctx: SpecialFormContext) -> CallableResult {
    Ok(ctx.eval_unary()?.expect_pair()?.car().into())
}

fn cdr(mut ctx: SpecialFormContext) -> CallableResult {
    Ok(ctx.eval_unary()?.expect_pair()?.cdr().into())
}

fn cons(mut ctx: SpecialFormContext) -> CallableResult {
    let (car, cdr) = ctx.eval_binary()?;
    let pair = Value::Pair(ctx.interpreter.pair_manager.pair(car, cdr)).source_mapped(ctx.range);
    Ok(pair.into())
}

fn list(mut ctx: SpecialFormContext) -> CallableResult {
    let operands = ctx.eval_variadic()?;
    Ok(ctx
        .interpreter
        .pair_manager
        .vec_to_list(operands.into())
        .into())
}

fn pair(mut ctx: SpecialFormContext) -> CallableResult {
    let operand = ctx.eval_unary()?;
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
