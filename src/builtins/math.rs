use crate::{
    interpreter::{ProcedureContext, ProcedureResult, RuntimeError, RuntimeErrorType},
    source_mapped::SourceMappable,
};

use super::util::number_args;

pub fn get_builtins() -> super::Builtins {
    vec![
        ("+", add),
        ("-", subtract),
        ("*", multiply),
        ("/", divide),
        ("remainder", remainder),
    ]
}

fn add(mut ctx: ProcedureContext) -> ProcedureResult {
    let mut result = 0.0;
    for number in number_args(&mut ctx)? {
        result += number
    }
    Ok(result.into())
}

fn subtract(mut ctx: ProcedureContext) -> ProcedureResult {
    let numbers = number_args(&mut ctx)?;
    if numbers.len() == 0 {
        return Err(RuntimeErrorType::WrongNumberOfArguments.source_mapped(ctx.combination.1));
    }
    let mut result = numbers[0];
    if numbers.len() == 1 {
        return Ok((-result).into());
    }
    for number in &numbers[1..] {
        result -= number
    }
    Ok(result.into())
}

fn multiply(mut ctx: ProcedureContext) -> ProcedureResult {
    let mut result = 1.0;
    for number in number_args(&mut ctx)? {
        result *= number
    }
    Ok(result.into())
}

fn divide(mut ctx: ProcedureContext) -> ProcedureResult {
    let numbers = number_args(&mut ctx)?;

    let divide_two = |a: f64, b: f64| -> Result<f64, RuntimeError> {
        if b == 0.0 {
            // Ideally we'd point at the specific argument that's zero, but this is good enough for now.
            return Err(RuntimeErrorType::DivisionByZero.source_mapped(ctx.combination.1));
        }
        Ok(a / b)
    };

    if numbers.len() == 0 {
        return Err(RuntimeErrorType::WrongNumberOfArguments.source_mapped(ctx.combination.1));
    }
    // Why are scheme's math operators so weird? This is how tryscheme.org's behaves, at least,
    // and I find it baffling.
    if numbers.len() == 1 {
        return Ok(divide_two(1.0, numbers[0])?.into());
    }
    let mut result = numbers[0];
    for &number in &numbers[1..] {
        result = divide_two(result, number)?;
    }
    Ok(result.into())
}

fn remainder(mut ctx: ProcedureContext) -> ProcedureResult {
    ctx.ensure_operands_len(2)?;
    let numbers = number_args(&mut ctx)?;
    Ok((numbers[0] % numbers[1]).into())
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
