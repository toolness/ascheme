use crate::{
    builtin_procedure::{BuiltinProcedureContext, BuiltinProcedureFn},
    builtins::Builtin,
    interpreter::{CallableResult, RuntimeError, RuntimeErrorType},
    source_mapped::SourceMappable,
    value::SourceValue,
};

use super::util::number_args;

pub fn get_builtins() -> super::Builtins {
    vec![
        Builtin::Procedure("+", BuiltinProcedureFn::NullaryVariadic(add)),
        Builtin::Procedure("-", BuiltinProcedureFn::UnaryVariadic(subtract)),
        Builtin::Procedure("*", BuiltinProcedureFn::NullaryVariadic(multiply)),
        Builtin::Procedure("/", BuiltinProcedureFn::UnaryVariadic(divide)),
        Builtin::Procedure("sqrt", BuiltinProcedureFn::Unary(sqrt)),
        Builtin::Procedure("remainder", BuiltinProcedureFn::Binary(remainder)),
    ]
}

fn sqrt(_ctx: BuiltinProcedureContext, value: &SourceValue) -> CallableResult {
    let number = value.expect_number()?;
    Ok(number.sqrt().into())
}

fn add(_ctx: BuiltinProcedureContext, operands: &[SourceValue]) -> CallableResult {
    let mut result = 0.0;
    for number in number_args(operands)? {
        result += number
    }
    Ok(result.into())
}

fn subtract(
    _ctx: BuiltinProcedureContext,
    first: &SourceValue,
    rest: &[SourceValue],
) -> CallableResult {
    let first = first.expect_number()?;
    let rest = number_args(rest)?;
    let mut result = first;
    if rest.is_empty() {
        return Ok((-result).into());
    }
    for number in &rest {
        result -= number
    }
    Ok(result.into())
}

fn multiply(_ctx: BuiltinProcedureContext, operands: &[SourceValue]) -> CallableResult {
    let mut result = 1.0;
    for number in number_args(operands)? {
        result *= number
    }
    Ok(result.into())
}

fn divide(
    ctx: BuiltinProcedureContext,
    first: &SourceValue,
    rest: &[SourceValue],
) -> CallableResult {
    let first = first.expect_number()?;
    let rest = number_args(rest)?;

    let divide_two = |a: f64, b: f64| -> Result<f64, RuntimeError> {
        if b == 0.0 {
            // Ideally we'd point at the specific argument that's zero, but this is good enough for now.
            return Err(RuntimeErrorType::DivisionByZero.source_mapped(ctx.range));
        }
        Ok(a / b)
    };

    // Why are scheme's math operators so weird? This is how tryscheme.org's behaves, at least,
    // and I find it baffling.
    if rest.is_empty() {
        return Ok(divide_two(1.0, first)?.into());
    }
    let mut result = first;
    for &number in &rest {
        result = divide_two(result, number)?;
    }
    Ok(result.into())
}

fn remainder(_ctx: BuiltinProcedureContext, a: &SourceValue, b: &SourceValue) -> CallableResult {
    Ok((a.expect_number()? % b.expect_number()?).into())
}

#[cfg(test)]
mod tests {
    use crate::{
        interpreter::RuntimeErrorType,
        test_util::{test_eval_err, test_eval_success},
    };

    #[test]
    fn basic_arithmetic_works() {
        // This is how try.scheme.org works, at least.
        test_eval_success("(+)", "0");
        test_eval_success("(*)", "1");

        test_eval_success("(+ 1 2)", "3");
        test_eval_success("(+ +1 2)", "3");
        test_eval_success("(+ -10 2)", "-8");
        test_eval_success("  (+ 1 2 (* 3 4)) ", "15");

        test_eval_success("(/ 2.0)", "0.5");
        test_eval_success("(/ 1.0 2.0)", "0.5");
        test_eval_success("(/ 1.0 2.0 2.0)", "0.25");
        test_eval_success("(/ 6 2)", "3");

        test_eval_success("(- 2)", "-2");
        test_eval_success("(- 5 2)", "3");
        test_eval_success("(- 5 2 1)", "2");
        test_eval_success("(- 5 2 10)", "-7");
    }

    #[test]
    fn remainder_works() {
        // From R5RS 6.2.5.
        test_eval_success("(remainder 13 4)", "1");
        test_eval_success("(remainder -13 4)", "-1");
        test_eval_success("(remainder 13 -4)", "1");
        test_eval_success("(remainder -13 -4)", "-1");
    }

    #[test]
    fn division_by_zero_raises_err() {
        test_eval_err("(/ 5 0)", RuntimeErrorType::DivisionByZero);
    }
}
