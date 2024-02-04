use std::rc::Rc;

use crate::{
    builtins,
    compound_procedure::{BoundProcedure, CompoundProcedure},
    environment::Environment,
    parser::{parse, Expression, ExpressionValue, ParseError, ParseErrorType},
    source_mapped::{SourceMappable, SourceMapped, SourceRange},
    source_mapper::{SourceId, SourceMapper},
    string_interner::{InternedString, StringInterner},
};

const MAX_STACK_SIZE: usize = 16;

#[derive(Debug)]
pub enum RuntimeErrorType {
    Parse(ParseErrorType),
    UnboundVariable(InternedString),
    MalformedExpression,
    MalformedSpecialForm,
    ExpectedNumber,
    ExpectedProcedure,
    ExpectedIdentifier,
    WrongNumberOfArguments,
    StackOverflow,
    // Unimplemented(&'static str),
}

pub type RuntimeError = SourceMapped<RuntimeErrorType>;

impl From<ParseError> for RuntimeError {
    fn from(value: ParseError) -> Self {
        RuntimeErrorType::Parse(value.0).source_mapped(value.1)
    }
}

impl SourceMapped<ExpressionValue> {
    pub fn expect_identifier(&self) -> Result<InternedString, RuntimeError> {
        if let ExpressionValue::Symbol(symbol) = &self.0 {
            Ok(symbol.clone())
        } else {
            Err(RuntimeErrorType::ExpectedIdentifier.source_mapped(self.1))
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Undefined,
    Number(f64),
    Procedure(Procedure),
}

pub struct ProcedureContext<'a> {
    pub interpreter: &'a mut Interpreter,
    pub combination: SourceMapped<&'a Rc<Vec<Expression>>>,
    pub operands: &'a [Expression],
}

#[derive(Debug, PartialEq, Clone)]
pub enum Procedure {
    Builtin(ProcedureFn, InternedString),
    Compound(CompoundProcedure),
}

pub type ProcedureResult = Result<ProcedureSuccess, RuntimeError>;

pub type ProcedureFn = fn(ProcedureContext) -> ProcedureResult;

pub struct TailCallContext {
    bound_procedure: BoundProcedure,
}

pub enum ProcedureSuccess {
    Value(Value),
    TailCall(TailCallContext),
}

impl From<Value> for ProcedureSuccess {
    fn from(value: Value) -> Self {
        ProcedureSuccess::Value(value)
    }
}

pub struct Interpreter {
    pub environment: Environment,
    pub string_interner: StringInterner,
    pub source_mapper: SourceMapper,
    pub tracing: bool,
    stack: Vec<SourceRange>,
}

impl Interpreter {
    pub fn new() -> Self {
        let source_mapper = SourceMapper::default();
        let mut string_interner = StringInterner::default();
        let mut environment = Environment::default();
        builtins::populate_environment(&mut environment, &mut string_interner);
        Interpreter {
            environment,
            string_interner,
            source_mapper,
            tracing: false,
            stack: vec![],
        }
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
        combination: SourceMapped<&Rc<Vec<Expression>>>,
        operands: &[Expression],
        source_range: SourceRange,
    ) -> ProcedureResult {
        if self.stack.len() >= MAX_STACK_SIZE {
            return Err(RuntimeErrorType::StackOverflow.source_mapped(combination.1));
        }
        self.stack.push(source_range);
        let ctx = ProcedureContext {
            interpreter: self,
            combination,
            operands,
        };
        let result = match procedure {
            Procedure::Builtin(builtin, _name) => builtin(ctx)?,
            Procedure::Compound(compound) => compound.call(ctx)?,
        };
        // Note that the stack won't unwind if an error occured above--this is so we can get a stack trace
        // afterwards. It's up to the caller to clean things up after an error.
        self.stack.pop();

        Ok(result)
    }

    pub fn try_bind_tail_call_context(
        &mut self,
        expression: &Expression,
    ) -> Result<Option<TailCallContext>, RuntimeError> {
        match &expression.0 {
            ExpressionValue::Combination(expressions) => {
                // TODO: A lot of this is duplicated from eval_expression, it'd be nice to consolidate
                // somehow.
                let Some(operator) = expressions.get(0) else {
                    return Err(RuntimeErrorType::MalformedExpression.source_mapped(expression.1));
                };
                let procedure = self.expect_procedure(operator)?;
                let combination = SourceMapped(expressions, expression.1);
                let operands = &expressions[1..];
                if self.tracing {
                    if let Some(lines) = self.source_mapper.trace(&combination.1) {
                        println!("Creating tail call context {}", lines.join("\n"));
                    }
                }
                let mut ctx = ProcedureContext {
                    interpreter: self,
                    combination,
                    operands,
                };
                match procedure {
                    Procedure::Compound(compound) => {
                        let bound_procedure = compound.bind(&mut ctx)?;
                        Ok(Some(TailCallContext { bound_procedure }))
                    }
                    _ => Ok(None),
                }
            }
            _ => Ok(None),
        }
    }

    pub fn eval_expression(&mut self, expression: &Expression) -> Result<Value, RuntimeError> {
        match &expression.0 {
            ExpressionValue::Number(number) => Ok(Value::Number(*number)),
            ExpressionValue::Symbol(identifier) => {
                if let Some(value) = self.environment.get(identifier) {
                    Ok(value)
                } else {
                    Err(RuntimeErrorType::UnboundVariable(identifier.clone())
                        .source_mapped(expression.1))
                }
            }
            ExpressionValue::Combination(expressions) => {
                let Some(operator) = expressions.get(0) else {
                    return Err(RuntimeErrorType::MalformedExpression.source_mapped(expression.1));
                };
                let procedure = self.expect_procedure(operator)?;
                let combination = SourceMapped(expressions, expression.1);
                let operands = &expressions[1..];
                if self.tracing {
                    if let Some(lines) = self.source_mapper.trace(&combination.1) {
                        println!("Evaluating {}", lines.join("\n"));
                    }
                }
                let mut result =
                    self.eval_procedure(procedure, combination, operands, operator.1)?;
                loop {
                    match result {
                        ProcedureSuccess::Value(value) => return Ok(value),
                        ProcedureSuccess::TailCall(tail_call_context) => {
                            result = tail_call_context.bound_procedure.call(self)?;
                        }
                    }
                }
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

    pub fn parse(&mut self, source_id: SourceId) -> Result<Vec<Expression>, ParseError> {
        let code = self.source_mapper.get_contents(source_id);
        parse(code, &mut self.string_interner, Some(source_id))
    }

    pub fn evaluate(&mut self, source_id: SourceId) -> Result<Value, RuntimeError> {
        match self.parse(source_id) {
            Ok(expressions) => self.eval_expressions(&expressions),
            Err(err) => Err(err.into()),
        }
    }

    pub fn traceback(&self) -> String {
        if self.stack.is_empty() {
            return "".to_string();
        }

        let mut lines =
            vec!["Traceback (excluding tail calls, most recent call last):".to_string()];

        for source_range in self.stack.iter() {
            if let Some(trace) = self.source_mapper.trace(source_range) {
                for line in trace {
                    lines.push(format!("  {}", line));
                }
            } else {
                lines.push("  <Unknown>".into());
            };
        }

        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use crate::interpreter::{Interpreter, Value};

    fn test_eval_success(code: &'static str, expected_value: &'static str) {
        let mut interpreter = Interpreter::new();
        let source_id = interpreter
            .source_mapper
            .add("<String>".into(), code.into());
        match interpreter.evaluate(source_id) {
            Ok(value) => {
                let string = match value {
                    Value::Undefined => "".to_string(),
                    Value::Number(num) => num.to_string(),
                    Value::Procedure(_) => {
                        unimplemented!("Converting procedure to string is unimplemented")
                    }
                };
                assert_eq!(string.as_str(), expected_value, "Evaluating code '{code}'");
            }
            Err(err) => {
                panic!("Evaluating code '{code}' raised error {err:?}");
            }
        }
    }

    #[test]
    fn trivial_expressions_work() {
        test_eval_success("5", "5");
    }

    #[test]
    fn basic_arithmetic_works() {
        // This is how try.scheme.org works, at least.
        test_eval_success("(+)", "0");
        test_eval_success("(*)", "1");

        test_eval_success("(+ 1 2)", "3");
        test_eval_success("  (+ 1 2 (* 3 4)) ", "15");
    }

    #[test]
    fn variable_definitions_work() {
        test_eval_success("(define x 3) x", "3");
        test_eval_success("(define x 3) (define y (+ x 1)) (+ x y)", "7");
    }

    #[test]
    fn compound_procedure_definitions_work() {
        test_eval_success("(define (x) 3)", "");
        test_eval_success("(define (x) 3) (x)", "3");
        test_eval_success("(define (add-three n) (+ 3 n)) (add-three 1)", "4");
    }

    #[test]
    fn lambda_definitions_work() {
        test_eval_success("(define x (lambda () 3))", "");
        test_eval_success("(define x (lambda () 3)) (x)", "3");
        test_eval_success("(define add-three (lambda (n) (+ 3 n))) (add-three 1)", "4");
    }

    #[test]
    fn compound_procedues_prefer_argument_values_to_globals() {
        test_eval_success(
            "
            (define n 5)
            (define (add-three n) (+ 3 n))
            (+ (add-three 1) n)
        ",
            "9",
        );
    }

    #[test]
    fn compound_procedues_use_lexical_scope() {
        test_eval_success(
            "
            (define (make-adder n)
              (define (add-n x) (+ x n))
              add-n
            )
            (define add-three (make-adder 3))
            (add-three 1)
        ",
            "4",
        );
    }
}
