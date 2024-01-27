use crate::{
    parser::{Expression, ExpressionValue},
    source_mapped::{SourceMappable, SourceMapped},
    string_interner::StringInterner,
};

#[derive(Debug)]
pub enum RuntimeErrorType {
    UnboundVariable,
    IllFormedExpression,
    InvalidOperator,
    Unimplemented(&'static str),
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
        let mut last_value: Value = Value::Undefined;
        for expression in self.expressions {
            last_value = match &expression.0 {
                ExpressionValue::Number(number) => Value::Number(*number),
                ExpressionValue::Symbol(_) => {
                    // TODO: Look up the symbol in the environment and return its value, if possible.
                    return Err(RuntimeErrorType::UnboundVariable.source_mapped(expression.1));
                }
                ExpressionValue::Combination(expressions) => {
                    let Some(operator) = expressions.get(0) else {
                        return Err(
                            RuntimeErrorType::IllFormedExpression.source_mapped(expression.1)
                        );
                    };
                    match operator.0 {
                        ExpressionValue::Number(_) => {
                            return Err(RuntimeErrorType::InvalidOperator.source_mapped(operator.1))
                        }
                        ExpressionValue::Symbol(symbol) => {
                            let add = self.interner.intern("+");
                            let multiply = self.interner.intern("*");
                            if symbol == add {
                                // TODO: Implement addition!
                                Value::Undefined
                            } else if symbol == multiply {
                                // TODO: Implement multiplication!
                                Value::Undefined
                            } else {
                                // TODO: Look up the symbol in the environment.
                                return Err(
                                    RuntimeErrorType::UnboundVariable.source_mapped(operator.1)
                                );
                            }
                        }
                        ExpressionValue::Combination(_) => {
                            return Err(RuntimeErrorType::Unimplemented(
                                "TODO: Implement combinations for operators",
                            )
                            .source_mapped(operator.1))
                        }
                    }
                }
            };
        }
        Ok(last_value)
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
