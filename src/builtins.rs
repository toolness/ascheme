use crate::{
    interpreter::{Interpreter, ProcedureFn, RuntimeError, Value},
    parser::Expression,
};

pub fn get_builtins() -> Vec<(&'static str, ProcedureFn)> {
    vec![("+", add), ("*", multiply)]
}

fn add(interpreter: &Interpreter, operands: &[Expression]) -> Result<Value, RuntimeError> {
    let mut result = 0.0;
    for expr in operands.iter() {
        let number = interpreter.expect_number(expr)?;
        result += number
    }
    Ok(Value::Number(result))
}

fn multiply(interpreter: &Interpreter, operands: &[Expression]) -> Result<Value, RuntimeError> {
    let mut result = 1.0;
    for expr in operands.iter() {
        let number = interpreter.expect_number(expr)?;
        result *= number
    }
    Ok(Value::Number(result))
}
