use crate::{
    interpreter::{Interpreter, ProcedureFn, RuntimeError, RuntimeErrorType, Value},
    parser::{Expression, ExpressionValue},
    source_mapped::{SourceMappable, SourceMapped},
};

pub fn get_builtins() -> Vec<(&'static str, ProcedureFn)> {
    vec![("+", add), ("*", multiply), ("define", define)]
}

fn add(interpreter: &mut Interpreter, operands: &[Expression]) -> Result<Value, RuntimeError> {
    let mut result = 0.0;
    for expr in operands.iter() {
        let number = interpreter.expect_number(expr)?;
        result += number
    }
    Ok(Value::Number(result))
}

fn multiply(interpreter: &mut Interpreter, operands: &[Expression]) -> Result<Value, RuntimeError> {
    let mut result = 1.0;
    for expr in operands.iter() {
        let number = interpreter.expect_number(expr)?;
        result *= number
    }
    Ok(Value::Number(result))
}

fn define(interpreter: &mut Interpreter, operands: &[Expression]) -> Result<Value, RuntimeError> {
    match operands.get(0) {
        Some(SourceMapped(ExpressionValue::Symbol(name), ..)) => {
            let value = interpreter.eval_expressions(&operands[1..])?;
            interpreter.define_environment_value(*name, value);
            Ok(Value::Undefined)
        }
        Some(SourceMapped(ExpressionValue::Combination(_), range)) => Err(
            RuntimeErrorType::Unimplemented("TODO implement compound procedure definitions")
                .source_mapped(*range),
        ),
        // TODO: Source map to the 'define' call somehow
        _ => Err(RuntimeErrorType::MalformedExpression.empty_source_map()),
    }
}
