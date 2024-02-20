use std::backtrace::Backtrace;

use colored::Colorize;

use crate::{
    interpreter::{ProcedureContext, ProcedureResult, RuntimeErrorType},
    source_mapped::SourceMappable,
    value::Value,
};

use super::eq::is_eq;

pub fn get_builtins() -> super::Builtins {
    vec![
        ("rust-backtrace", rust_backtrace),
        ("stats", stats),
        ("gc", gc),
        ("test-eq", test_eq),
        ("assert", assert),
        ("print-and-eval", print_and_eval),
        ("track-call-stats", track_call_stats),
    ]
}

fn stats(ctx: ProcedureContext) -> ProcedureResult {
    ctx.interpreter.print_stats();
    Ok(Value::Undefined.into())
}

fn gc(ctx: ProcedureContext) -> ProcedureResult {
    let objs_found_in_cycles = ctx.interpreter.gc(true);
    Ok((objs_found_in_cycles as f64).into())
}

fn print_and_eval(ctx: ProcedureContext) -> ProcedureResult {
    if ctx.operands.len() != 1 {
        return Err(RuntimeErrorType::WrongNumberOfArguments.source_mapped(ctx.combination.1));
    }
    let operand_repr = ctx.operands[0].to_string();
    let value = ctx.interpreter.eval_expression(&ctx.operands[0])?;
    ctx.interpreter
        .printer
        .println(format!("{} = {}", operand_repr, value));
    Ok(value.into())
}

fn assert(ctx: ProcedureContext) -> ProcedureResult {
    if ctx.operands.len() != 1 {
        return Err(RuntimeErrorType::WrongNumberOfArguments.source_mapped(ctx.combination.1));
    }
    let value = ctx.interpreter.eval_expression(&ctx.operands[0])?.0;
    if !value.as_bool() {
        Err(RuntimeErrorType::AssertionFailure.source_mapped(ctx.combination.1))
    } else {
        Ok(Value::Undefined.into())
    }
}

fn test_eq(mut ctx: ProcedureContext) -> ProcedureResult {
    if ctx.operands.len() != 2 {
        return Err(RuntimeErrorType::WrongNumberOfArguments.source_mapped(ctx.combination.1));
    }
    let operand_0_repr = ctx.operands[0].to_string();
    let operand_1_repr = ctx.operands[1].to_string();

    let msg = if is_eq(&mut ctx.interpreter, &ctx.operands[0], &ctx.operands[1])? {
        "OK".green()
    } else {
        "ERR".red()
    };

    ctx.interpreter
        .printer
        .println(format!("{msg} {operand_0_repr} = {operand_1_repr}"));

    Ok(Value::Undefined.into())
}

fn rust_backtrace(ctx: ProcedureContext) -> ProcedureResult {
    let location = ctx
        .interpreter
        .source_mapper
        .trace(&ctx.combination.1)
        .join("\n");
    let backtrace = Backtrace::force_capture();
    ctx.interpreter
        .printer
        .println(format!("Rust backtrace at {location}\n{backtrace}"));
    ctx.interpreter
        .eval_expressions_in_tail_context(ctx.operands)
}

fn track_call_stats(ctx: ProcedureContext) -> ProcedureResult {
    if ctx.operands.len() != 1 {
        return Err(RuntimeErrorType::WrongNumberOfArguments.source_mapped(ctx.combination.1));
    }
    ctx.interpreter.start_tracking_stats();
    let value = ctx.interpreter.eval_expression(&ctx.operands[0])?;
    if let Some(stats) = ctx.interpreter.take_tracked_stats() {
        println!("{stats:#?}");
    }
    Ok(value.into())
}

#[cfg(test)]
mod tests {
    use crate::{
        interpreter::RuntimeErrorType,
        test_util::{test_eval_err, test_eval_success},
    };

    #[test]
    fn assert_does_nothing_when_operand_is_true() {
        test_eval_success("(assert #t)", "");
        test_eval_success("(assert (+ 0 0))", "");
    }

    #[test]
    fn assert_errors_when_operand_is_false() {
        test_eval_err("(assert #f)", RuntimeErrorType::AssertionFailure);
    }
}
