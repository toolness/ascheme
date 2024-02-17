use std::backtrace::Backtrace;

use crate::{
    interpreter::{ProcedureContext, ProcedureResult, RuntimeErrorType},
    source_mapped::SourceMappable,
    value::Value,
};

use super::eq::is_eq;

pub fn stats(ctx: ProcedureContext) -> ProcedureResult {
    ctx.interpreter.print_stats();
    Ok(Value::Undefined.into())
}

pub fn gc(ctx: ProcedureContext) -> ProcedureResult {
    let objs_found_in_cycles = ctx.interpreter.gc(true);
    Ok((objs_found_in_cycles as f64).into())
}

pub fn print_and_eval(ctx: ProcedureContext) -> ProcedureResult {
    if ctx.operands.len() != 1 {
        return Err(RuntimeErrorType::WrongNumberOfArguments.source_mapped(ctx.combination.1));
    }
    let operand_repr = ctx.operands[0].to_string();
    let value = ctx.interpreter.eval_expression(&ctx.operands[0])?;
    println!("{} = {}", operand_repr, value);
    Ok(value.into())
}

pub fn test_eq(mut ctx: ProcedureContext) -> ProcedureResult {
    if ctx.operands.len() != 2 {
        return Err(RuntimeErrorType::WrongNumberOfArguments.source_mapped(ctx.combination.1));
    }
    let operand_0_repr = ctx.operands[0].to_string();
    let operand_1_repr = ctx.operands[1].to_string();

    if is_eq(&mut ctx.interpreter, &ctx.operands[0], &ctx.operands[1])? {
        println!("OK {} = {}", operand_0_repr, operand_1_repr);
    } else {
        println!("ERR {} != {}", operand_0_repr, operand_1_repr);
    }

    Ok(Value::Undefined.into())
}

pub fn rust_backtrace(ctx: ProcedureContext) -> ProcedureResult {
    println!(
        "Rust backtrace at {}",
        ctx.interpreter
            .source_mapper
            .trace(&ctx.combination.1)
            .join("\n")
    );
    println!("{}", Backtrace::force_capture());
    ctx.interpreter
        .eval_expressions_in_tail_context(ctx.operands)
}
