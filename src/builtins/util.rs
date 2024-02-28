use crate::interpreter::{CallableContext, RuntimeError};

pub fn number_args(ctx: &mut CallableContext) -> Result<Vec<f64>, RuntimeError> {
    let mut numbers = Vec::with_capacity(ctx.operands.len());
    for expr in ctx.operands.iter() {
        numbers.push(ctx.interpreter.expect_number(expr)?);
    }
    Ok(numbers)
}
