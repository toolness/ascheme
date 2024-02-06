use std::{fmt::Display, rc::Rc, sync::mpsc::Receiver};

use crate::{
    builtins,
    compound_procedure::{BoundProcedure, CompoundProcedure},
    environment::Environment,
    parser::{parse, Expression, ExpressionValue, ParseError, ParseErrorType},
    source_mapped::{SourceMappable, SourceMapped, SourceRange},
    source_mapper::{SourceId, SourceMapper},
    string_interner::{InternedString, StringInterner},
};

const DEFAULT_MAX_STACK_SIZE: usize = 32;

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
    KeyboardInterrupt,
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
    Boolean(bool),
    Procedure(Procedure),
}

impl Value {
    /// From R5RS 6.3.1:
    ///
    /// > Of all the standard Scheme values, only `#f` counts as false
    /// > in conditional expressions. Except for `#f`, all standard
    /// > Scheme values, including `#t`, pairs, the empty list, symbols,
    /// > numbers, strings, vectors, and procedures, count as true.
    pub fn as_bool(&self) -> bool {
        match self {
            Value::Boolean(false) => false,
            _ => true,
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Undefined => write!(f, ""),
            Value::Number(value) => write!(f, "{}", value),
            Value::Boolean(boolean) => write!(f, "{}", if *boolean { "#t" } else { "#f" }),
            Value::Procedure(Procedure::Builtin(_, name)) => {
                write!(f, "#<builtin procedure {}>", name.as_ref())
            }
            Value::Procedure(Procedure::Compound(compound)) => write!(
                f,
                "#<procedure{} #{}>",
                match &compound.name {
                    Some(name) => format!(" {}", name.as_ref()),
                    None => format!(""),
                },
                compound.id()
            ),
        }
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Boolean(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::Number(value)
    }
}

impl<T: Into<Value>> From<T> for ProcedureSuccess {
    fn from(value: T) -> Self {
        ProcedureSuccess::Value(value.into())
    }
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

pub struct Interpreter {
    pub environment: Environment,
    pub string_interner: StringInterner,
    pub source_mapper: SourceMapper,
    pub tracing: bool,
    pub max_stack_size: usize,
    pub keyboard_interrupt_channel: Option<Receiver<()>>,
    next_id: u32,
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
            max_stack_size: DEFAULT_MAX_STACK_SIZE,
            keyboard_interrupt_channel: None,
            next_id: 1,
            stack: vec![],
        }
    }

    pub fn new_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
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
        if self.stack.len() >= self.max_stack_size {
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

    fn try_bind_tail_call_context(
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
                match procedure {
                    Procedure::Compound(compound) => {
                        if self.tracing {
                            println!(
                                "Creating tail call context {}",
                                self.source_mapper.trace(&combination.1).join("\n")
                            );
                        }
                        let mut ctx = ProcedureContext {
                            interpreter: self,
                            combination,
                            operands,
                        };
                        let bound_procedure = compound.bind(&mut ctx)?;
                        Ok(Some(TailCallContext { bound_procedure }))
                    }
                    _ => Ok(None),
                }
            }
            _ => Ok(None),
        }
    }

    pub fn eval_expression_in_tail_context(&mut self, expression: &Expression) -> ProcedureResult {
        if let Some(tail_call_context) = self.try_bind_tail_call_context(expression)? {
            Ok(ProcedureSuccess::TailCall(tail_call_context))
        } else {
            self.lazy_eval_expression(expression)
        }
    }

    fn lazy_eval_expression(&mut self, expression: &Expression) -> ProcedureResult {
        match &expression.0 {
            ExpressionValue::Number(number) => Ok(Value::Number(*number).into()),
            ExpressionValue::Boolean(boolean) => Ok(Value::Boolean(*boolean).into()),
            ExpressionValue::Symbol(identifier) => {
                if let Some(value) = self.environment.get(identifier) {
                    Ok(value.into())
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
                    println!(
                        "Evaluating {}",
                        self.source_mapper.trace(&combination.1).join("\n")
                    );
                }
                self.eval_procedure(procedure, combination, operands, operator.1)
            }
        }
    }

    pub fn eval_expression(&mut self, expression: &Expression) -> Result<Value, RuntimeError> {
        let mut result = self.lazy_eval_expression(expression)?;
        loop {
            if let Some(channel) = &self.keyboard_interrupt_channel {
                if channel.try_recv().is_ok() {
                    return Err(RuntimeErrorType::KeyboardInterrupt.source_mapped(expression.1));
                }
            }
            match result {
                ProcedureSuccess::Value(value) => return Ok(value),
                ProcedureSuccess::TailCall(tail_call_context) => {
                    result = tail_call_context.bound_procedure.call(self)?;
                }
            }
        }
    }

    pub fn eval_expressions_in_tail_context(
        &mut self,
        expressions: &[Expression],
    ) -> Result<ProcedureSuccess, RuntimeError> {
        if expressions.len() == 0 {
            return Ok(Value::Undefined.into());
        }

        if expressions.len() > 1 {
            self.eval_expressions(&expressions[0..expressions.len() - 1])?;
        }

        let last_expression = &expressions[expressions.len() - 1];
        self.eval_expression_in_tail_context(last_expression)
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
        self.stack.clear();
        self.environment.clear_lexical_scopes();
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
            for line in self.source_mapper.trace(source_range) {
                lines.push(format!("  {}", line));
            }
        }

        lines.join("\n")
    }
}
