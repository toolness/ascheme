use std::collections::HashMap;

use crate::{
    builtins::get_builtins,
    parser::{Expression, ExpressionValue},
    source_mapped::{SourceMappable, SourceMapped},
    string_interner::{InternedString, StringInterner},
};

#[derive(Debug)]
pub enum RuntimeErrorType {
    UnboundVariable,
    MalformedExpression,
    ExpectedNumber,
    ExpectedProcedure,
    Unimplemented(&'static str),
}

pub type RuntimeError = SourceMapped<RuntimeErrorType>;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Value {
    Undefined,
    Number(f64),
    Procedure(Procedure),
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Procedure {
    Builtin(ProcedureFn),
}

pub type ProcedureFn = fn(&mut Interpreter, &[Expression]) -> Result<Value, RuntimeError>;

pub type Environment = HashMap<InternedString, Value>;

pub struct Interpreter {
    environment: Environment,
}

impl Interpreter {
    pub fn define_environment_value(&mut self, name: InternedString, value: Value) {
        self.environment.insert(name, value);
    }

    pub fn expect_number(&mut self, expression: &Expression) -> Result<f64, RuntimeError> {
        if let Value::Number(number) = self.eval_expression(&expression)? {
            Ok(number)
        } else {
            Err(RuntimeErrorType::ExpectedNumber.source_mapped(expression.1))
        }
    }

    fn expect_procedure(&mut self, expression: &Expression) -> Result<Procedure, RuntimeError> {
        if let Value::Procedure(procedure) = self.eval_expression(&expression)? {
            Ok(procedure)
        } else {
            Err(RuntimeErrorType::ExpectedProcedure.source_mapped(expression.1))
        }
    }

    fn eval_procedure(
        &mut self,
        procedure: Procedure,
        operands: &[Expression],
    ) -> Result<Value, RuntimeError> {
        match procedure {
            Procedure::Builtin(builtin) => builtin(self, operands),
        }
    }

    fn eval_expression(&mut self, expression: &Expression) -> Result<Value, RuntimeError> {
        match &expression.0 {
            ExpressionValue::Number(number) => Ok(Value::Number(*number)),
            ExpressionValue::Symbol(identifier) => {
                if let Some(value) = self.environment.get(identifier) {
                    Ok(*value)
                } else {
                    Err(RuntimeErrorType::UnboundVariable.source_mapped(expression.1))
                }
            }
            ExpressionValue::Combination(expressions) => {
                let Some(operator) = expressions.get(0) else {
                    return Err(RuntimeErrorType::MalformedExpression.source_mapped(expression.1));
                };
                let procedure = self.expect_procedure(operator)?;
                self.eval_procedure(procedure, &expressions[1..])
            }
        }
    }

    pub fn eval_expressions(&mut self, expressions: &[Expression]) -> Result<Value, RuntimeError> {
        let mut last_value: Value = Value::Undefined;
        for expression in expressions {
            last_value = self.eval_expression(expression)?;
        }
        Ok(last_value)
    }

    pub fn evaluate(
        expressions: &Vec<Expression>,
        interner: &mut StringInterner,
    ) -> Result<Value, RuntimeError> {
        let mut environment: Environment = Default::default();
        for (name, builtin) in get_builtins() {
            environment.insert(
                interner.intern(name),
                Value::Procedure(Procedure::Builtin(builtin)),
            );
        }
        let mut interpreter = Interpreter { environment };
        interpreter.eval_expressions(expressions)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        interpreter::{Interpreter, Value},
        parser::parse,
        string_interner::StringInterner,
    };

    fn test_eval_success(code: &'static str, expected_value: &'static str) {
        let mut interner = StringInterner::default();
        match parse(code, &mut interner) {
            Ok(expressions) => match Interpreter::evaluate(&expressions, &mut interner) {
                Ok(value) => {
                    let string = match value {
                        Value::Undefined => "".to_string(),
                        Value::Number(num) => num.to_string(),
                        Value::Procedure(_) => unimplemented!(),
                    };
                    assert_eq!(string.as_str(), expected_value, "Evaluating code '{code}'");
                }
                Err(err) => {
                    panic!("Evaluating code '{code}' raised error {err:?}");
                }
            },
            Err(err) => {
                panic!("Parsing code '{code}' raised error {err:?}");
            }
        }
    }

    #[test]
    fn trivial_expressions_work() {
        test_eval_success("5", "5");
    }

    #[test]
    fn basic_arithmetic_works() {
        test_eval_success("(+ 1 2)", "3");
        test_eval_success("  (+ 1 2 (* 3 4)) ", "15");
    }

    #[test]
    fn variable_definitions_work() {
        test_eval_success("(define x 3) x", "3");
        test_eval_success("(define x 3) (define y (+ x 1)) (+ x y)", "7");
    }
}
