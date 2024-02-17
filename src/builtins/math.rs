use crate::{
    interpreter::{ProcedureContext, ProcedureResult, RuntimeError, RuntimeErrorType},
    source_mapped::SourceMappable,
};

use super::util::number_args;

pub fn add(mut ctx: ProcedureContext) -> ProcedureResult {
    let mut result = 0.0;
    for number in number_args(&mut ctx)? {
        result += number
    }
    Ok(result.into())
}

pub fn subtract(mut ctx: ProcedureContext) -> ProcedureResult {
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

pub fn multiply(mut ctx: ProcedureContext) -> ProcedureResult {
    let mut result = 1.0;
    for number in number_args(&mut ctx)? {
        result *= number
    }
    Ok(result.into())
}

pub fn divide(mut ctx: ProcedureContext) -> ProcedureResult {
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
