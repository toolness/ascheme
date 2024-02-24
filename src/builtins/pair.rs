use crate::{
    interpreter::{ProcedureContext, ProcedureResult, RuntimeError},
    pair::Pair,
    source_mapped::SourceMappable,
    value::{SourceValue, Value},
};

use super::Builtins;

pub fn get_builtins() -> Builtins {
    vec![
        ("set-car!", set_car),
        ("set-cdr!", set_cdr),
        ("cons", cons),
        ("car", car),
        ("cdr", cdr),
    ]
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

fn car(mut ctx: ProcedureContext) -> ProcedureResult {
    Ok(ctx.eval_unary()?.expect_pair()?.car().into())
}

fn cdr(mut ctx: ProcedureContext) -> ProcedureResult {
    Ok(ctx.eval_unary()?.expect_pair()?.cdr().into())
}

fn cons(mut ctx: ProcedureContext) -> ProcedureResult {
    let (car, cdr) = ctx.eval_binary()?;
    let pair =
        Value::Pair(ctx.interpreter.pair_manager.pair(car, cdr)).source_mapped(ctx.combination.1);
    Ok(pair.into())
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
}
