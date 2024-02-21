use std::backtrace::Backtrace;

use colored::Colorize;

use crate::{
    interpreter::{ProcedureContext, ProcedureResult, RuntimeErrorType},
    source_mapped::SourceMappable,
};

use super::eq::is_eq;

pub fn get_builtins() -> super::Builtins {
    vec![
        ("rust-backtrace", rust_backtrace),
        ("stats", stats),
        ("gc", gc),
        ("gc-verbose", gc_verbose),
        ("test-eq", test_eq),
        ("assert", assert),
        ("print-and-eval", print_and_eval),
        ("track-call-stats", track_call_stats),
    ]
}

fn stats(ctx: ProcedureContext) -> ProcedureResult {
    ctx.interpreter.print_stats();
    ctx.undefined()
}

fn gc(ctx: ProcedureContext) -> ProcedureResult {
    let objs_found_in_cycles = ctx.interpreter.gc(false);
    Ok((objs_found_in_cycles as f64).into())
}

fn gc_verbose(ctx: ProcedureContext) -> ProcedureResult {
    let objs_found_in_cycles = ctx.interpreter.gc(true);
    Ok((objs_found_in_cycles as f64).into())
}

fn print_and_eval(ctx: ProcedureContext) -> ProcedureResult {
    ctx.ensure_operands_len(1)?;
    let operand_repr = ctx.operands[0].to_string();
    let value = ctx.interpreter.eval_expression(&ctx.operands[0])?;
    ctx.interpreter
        .printer
        .println(format!("{} = {}", operand_repr, value));
    Ok(value.into())
}

fn assert(mut ctx: ProcedureContext) -> ProcedureResult {
    let value = ctx.eval_unary()?;
    if !value.0.as_bool() {
        Err(RuntimeErrorType::AssertionFailure.source_mapped(ctx.combination.1))
    } else {
        ctx.undefined()
    }
}

fn test_eq(mut ctx: ProcedureContext) -> ProcedureResult {
    ctx.ensure_operands_len(2)?;
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

    ctx.undefined()
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

fn track_call_stats(mut ctx: ProcedureContext) -> ProcedureResult {
    ctx.interpreter.start_tracking_stats();
    let value = ctx.eval_unary()?;
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
