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
        ("test-repr", test_repr),
        ("assert", assert),
        ("print-and-eval", print_and_eval),
        ("track-stats", track_stats),
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
    for (i, operand) in ctx.operands.iter().enumerate() {
        let operand_repr = operand.to_string();
        let value = ctx.interpreter.eval_expression(&operand)?;
        let end = if i == ctx.operands.len() - 1 {
            "\n"
        } else {
            ", "
        };
        ctx.interpreter
            .printer
            .print(format!("{} = {value}{end}", operand_repr.blue()));
    }
    ctx.undefined()
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

fn test_repr(ctx: ProcedureContext) -> ProcedureResult {
    ctx.ensure_operands_len(2)?;
    let operand_0_repr = ctx.operands[0].to_string();
    let operand_0_value = ctx.interpreter.eval_expression(&ctx.operands[0])?;
    let operand_1_value = ctx.interpreter.eval_expression(&ctx.operands[1])?;
    let operand_0_value_repr = operand_0_value.to_string();
    let operand_1_value_repr = operand_1_value.to_string();

    let msg = if operand_0_value_repr == operand_1_value_repr {
        format!("{} {operand_0_repr} = {operand_1_value_repr}", "OK".green())
    } else {
        format!(
            "{} {operand_0_repr} = {operand_0_value_repr} ≠ {operand_1_value_repr}",
            "ERR".red()
        )
    };

    ctx.interpreter.printer.println(msg);
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

fn track_stats(mut ctx: ProcedureContext) -> ProcedureResult {
    ctx.ensure_operands_len(1)?;
    let repr = ctx.operands[0].to_string();
    ctx.interpreter.start_tracking_stats();
    let result = ctx.eval_unary();
    println!("Statistics for evaluation of {}\n", repr.blue());
    if let Some(stats) = ctx.interpreter.take_tracked_stats() {
        ctx.interpreter.printer.println(stats.as_table());
    }
    result?;
    ctx.undefined()
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
