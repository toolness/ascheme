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
    ExpectedNumber,
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
    fn expect_number(&mut self, expression: &Expression) -> Result<f64, RuntimeError> {
        if let Value::Number(number) = self.eval_expression(&expression)? {
            Ok(number)
        } else {
            Err(RuntimeErrorType::ExpectedNumber.source_mapped(expression.1))
        }
    }

    fn eval_expression(&mut self, expression: &Expression) -> Result<Value, RuntimeError> {
        match &expression.0 {
            ExpressionValue::Number(number) => Ok(Value::Number(*number)),
            ExpressionValue::Symbol(_) => {
                // TODO: Look up the symbol in the environment and return its value, if possible.
                return Err(RuntimeErrorType::UnboundVariable.source_mapped(expression.1));
            }
            ExpressionValue::Combination(expressions) => {
                let Some(operator) = expressions.get(0) else {
                    return Err(RuntimeErrorType::IllFormedExpression.source_mapped(expression.1));
                };
                match operator.0 {
                    ExpressionValue::Number(_) => {
                        return Err(RuntimeErrorType::InvalidOperator.source_mapped(operator.1))
                    }
                    ExpressionValue::Symbol(symbol) => {
                        let add = self.interner.intern("+");
                        let multiply = self.interner.intern("*");
                        if symbol == add {
                            let mut result = 0.0;
                            for expr in expressions[1..].iter() {
                                let number = self.expect_number(expr)?;
                                result += number
                            }
                            Ok(Value::Number(result))
                        } else if symbol == multiply {
                            let mut result = 1.0;
                            for expr in expressions[1..].iter() {
                                let number = self.expect_number(expr)?;
                                result *= number
                            }
                            Ok(Value::Number(result))
                        } else {
                            // TODO: Look up the symbol in the environment.
                            return Err(RuntimeErrorType::UnboundVariable.source_mapped(operator.1));
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
        }
    }

    fn eval(&mut self) -> Result<Value, RuntimeError> {
        let mut last_value: Value = Value::Undefined;
        for expression in self.expressions {
            last_value = self.eval_expression(expression)?;
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
