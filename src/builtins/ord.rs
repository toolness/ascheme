use std::f64::INFINITY;

use crate::interpreter::{ProcedureContext, ProcedureResult};

use super::util::number_args;

pub fn less_than(mut ctx: ProcedureContext) -> ProcedureResult {
    let mut latest: f64 = -INFINITY;
    for number in number_args(&mut ctx)? {
        if number <= latest {
            return Ok(false.into());
        }
        latest = number;
    }
    Ok(true.into())
}

pub fn less_than_or_equal_to(mut ctx: ProcedureContext) -> ProcedureResult {
    let mut latest: f64 = -INFINITY;
    for number in number_args(&mut ctx)? {
        if number < latest {
            return Ok(false.into());
        }
        latest = number;
    }
    Ok(true.into())
}

pub fn greater_than(mut ctx: ProcedureContext) -> ProcedureResult {
    let mut latest: f64 = INFINITY;
    for number in number_args(&mut ctx)? {
        if number >= latest {
            return Ok(false.into());
        }
        latest = number;
    }
    Ok(true.into())
}

pub fn greater_than_or_equal_to(mut ctx: ProcedureContext) -> ProcedureResult {
    let mut latest: f64 = INFINITY;
    for number in number_args(&mut ctx)? {
        if number > latest {
            return Ok(false.into());
        }
        latest = number;
    }
    Ok(true.into())
}

pub fn numeric_eq(mut ctx: ProcedureContext) -> ProcedureResult {
    let numbers = number_args(&mut ctx)?;
    if numbers.len() < 2 {
        Ok(true.into())
    } else {
        let number = numbers[0];
        for other_number in &numbers[1..] {
            if *other_number != number {
                return Ok(false.into());
            }
        }
        Ok(true.into())
    }
}
