use std::collections::HashMap;

use crate::{
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
    // Unimplemented(&'static str),
}

pub type RuntimeError = SourceMapped<RuntimeErrorType>;

#[derive(Debug)]
pub enum Value {
    Undefined,
    Number(f64),
    Procedure(Procedure),
}

#[derive(Debug)]
pub enum Procedure {
    Builtin(InternedString),
}

type ProcedureFn = dyn Fn(&Interpreter, &[Expression]) -> Result<Value, RuntimeError>;

pub struct Interpreter<'a> {
    builtins: HashMap<InternedString, Box<ProcedureFn>>,
    expressions: &'a Vec<Expression>,
}

impl<'a> Interpreter<'a> {
    fn expect_number(&self, expression: &Expression) -> Result<f64, RuntimeError> {
        if let Value::Number(number) = self.eval_expression(&expression)? {
            Ok(number)
        } else {
            Err(RuntimeErrorType::ExpectedNumber.source_mapped(expression.1))
        }
    }

    fn expect_procedure(&self, expression: &Expression) -> Result<Procedure, RuntimeError> {
        if let Value::Procedure(procedure) = self.eval_expression(&expression)? {
            Ok(procedure)
        } else {
            Err(RuntimeErrorType::ExpectedProcedure.source_mapped(expression.1))
        }
    }

    fn eval_procedure(
        &self,
        procedure: Procedure,
        operands: &[Expression],
    ) -> Result<Value, RuntimeError> {
        match procedure {
            Procedure::Builtin(name) => {
                let builtin = self.builtins.get(&name).expect("Builtin should exist");
                builtin(self, operands)
            }
        }
    }

    fn eval_expression(&self, expression: &Expression) -> Result<Value, RuntimeError> {
        match &expression.0 {
            ExpressionValue::Number(number) => Ok(Value::Number(*number)),
            ExpressionValue::Symbol(identifier) => {
                if self.builtins.contains_key(&identifier) {
                    Ok(Value::Procedure(Procedure::Builtin(*identifier)))
                } else {
                    // TODO: Look up the symbol in the environment and return its value, if possible.
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
            builtins: make_builtins(interner),
            expressions,
        };
        interpreter.eval()
    }
}

fn make_builtins(interner: &mut StringInterner) -> HashMap<InternedString, Box<ProcedureFn>> {
    let mut builtins: HashMap<InternedString, Box<ProcedureFn>> = HashMap::new();
    builtins.insert(interner.intern("+"), Box::new(add));
    builtins.insert(interner.intern("*"), Box::new(multiply));
    builtins
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
