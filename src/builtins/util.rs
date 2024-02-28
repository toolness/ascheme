use crate::{interpreter::RuntimeError, value::SourceValue};

pub fn number_args(operands: &[SourceValue]) -> Result<Vec<f64>, RuntimeError> {
    let mut numbers = Vec::with_capacity(operands.len());
    for operand in operands.iter() {
        numbers.push(operand.expect_number()?);
    }
    Ok(numbers)
}
