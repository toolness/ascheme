use crate::{
    interpreter::{ProcedureContext, ProcedureResult, RuntimeErrorType},
    source_mapped::SourceMappable,
};

pub fn and(ctx: ProcedureContext) -> ProcedureResult {
    let mut latest = true.into();
    for (i, operand) in ctx.operands.iter().enumerate() {
        if i == ctx.operands.len() - 1 {
            return ctx.interpreter.eval_expression_in_tail_context(operand);
        }
        latest = ctx.interpreter.eval_expression(operand)?.0;
        if !latest.as_bool() {
            break;
        }
    }
    Ok(latest.into())
}

pub fn or(ctx: ProcedureContext) -> ProcedureResult {
    let mut latest = false.into();
    for (i, operand) in ctx.operands.iter().enumerate() {
        if i == ctx.operands.len() - 1 {
            return ctx.interpreter.eval_expression_in_tail_context(operand);
        }
        latest = ctx.interpreter.eval_expression(operand)?.0;
        if latest.as_bool() {
            break;
        }
    }
    Ok(latest.into())
}

pub fn not(ctx: ProcedureContext) -> ProcedureResult {
    if ctx.operands.len() != 1 {
        return Err(RuntimeErrorType::WrongNumberOfArguments.source_mapped(ctx.combination.1));
    }
    let result = ctx.interpreter.eval_expression(&ctx.operands[0])?.0;
    Ok((!result.as_bool()).into())
}

#[cfg(test)]
mod tests {
    use crate::test_util::test_eval_success;

    #[test]
    fn and_works() {
        test_eval_success("(and)", "#t");
        test_eval_success("(and 1)", "1");
        test_eval_success("(and 1 2)", "2");
        test_eval_success("(and #f 2)", "#f");
        test_eval_success("(and #f lololol)", "#f");
    }

    #[test]
    fn or_works() {
        test_eval_success("(or)", "#f");
        test_eval_success("(or 1)", "1");
        test_eval_success("(or 1 2)", "1");
        test_eval_success("(or #f 2)", "2");
        test_eval_success("(or 1 lololol)", "1");
    }

    #[test]
    fn not_works() {
        test_eval_success("(not #t)", "#f");
        test_eval_success("(not 0)", "#f");
        test_eval_success("(not #f)", "#t");
        test_eval_success("(not (= 3 1))", "#t");
    }
}
