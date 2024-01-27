use crate::{
    parser::{Expression, ExpressionValue},
    source_mapped::{SourceMappable, SourceMapped},
    string_interner::StringInterner,
};

#[derive(Debug)]
pub enum RuntimeErrorType {
    UnknownIdentifier,
}

pub type RuntimeError = SourceMapped<RuntimeErrorType>;

#[derive(Debug)]
pub enum Value {
    Undefined,
    Number(f64),
}

pub struct Interpreter<'a> {
    expressions: &'a Vec<Expression>,
    interner: &'a mut StringInterner,
}

impl<'a> Interpreter<'a> {
    fn eval(&mut self) -> Result<Value, RuntimeError> {
        for expression in self.expressions {
            let value = match &expression.0 {
                ExpressionValue::Number(number) => Value::Number(*number),
                ExpressionValue::Symbol(_) => {
                    // TODO: Look up the symbol in the environment and return its value, if possible.
                    return Err(RuntimeErrorType::UnknownIdentifier.source_mapped(expression.1));
                }
                ExpressionValue::Combination(expressions) => {
                    todo!("Implement combinations")
                }
            };
        }
        Ok(Value::Undefined)
    }

    pub fn evaluate(
        expressions: &Vec<Expression>,
        interner: &'a mut StringInterner,
    ) -> Result<Value, RuntimeError> {
        let mut interpreter = Interpreter {
            expressions,
            interner,
        };
        interpreter.eval()
    }
}
