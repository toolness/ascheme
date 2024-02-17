use std::{ops::Deref, rc::Rc, sync::mpsc::Receiver};

use crate::{
    builtins,
    compound_procedure::{BoundProcedure, CompoundProcedure},
    environment::Environment,
    gc::Visitor,
    gc_rooted::GCRootManager,
    pair::PairManager,
    parser::{parse, ParseError, ParseErrorType},
    source_mapped::{SourceMappable, SourceMapped, SourceRange},
    source_mapper::{SourceId, SourceMapper},
    string_interner::{InternedString, StringInterner},
    value::{SourceValue, Value},
};

const DEFAULT_MAX_STACK_SIZE: usize = 128;

#[derive(Debug)]
pub enum RuntimeErrorType {
    Parse(ParseErrorType),
    UnboundVariable(InternedString),
    MalformedExpression,
    MalformedSpecialForm,
    ExpectedNumber,
    ExpectedProcedure,
    ExpectedIdentifier,
    ExpectedPair,
    WrongNumberOfArguments,
    StackOverflow,
    KeyboardInterrupt,
    DivisionByZero,
    // Unimplemented(&'static str),
}

pub type RuntimeError = SourceMapped<RuntimeErrorType>;

impl From<ParseError> for RuntimeError {
    fn from(value: ParseError) -> Self {
        RuntimeErrorType::Parse(value.0).source_mapped(value.1)
    }
}

impl<T: Into<SourceValue>> From<T> for ProcedureSuccess {
    fn from(value: T) -> Self {
        ProcedureSuccess::Value(value.into())
    }
}

pub struct ProcedureContext<'a> {
    pub interpreter: &'a mut Interpreter,
    pub combination: SourceMapped<&'a Rc<Vec<SourceValue>>>,
    pub operands: &'a [SourceValue],
}

#[derive(Debug, Clone)]
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
    Value(SourceValue),
    TailCall(TailCallContext),
}

pub struct Interpreter {
    pub environment: Environment,
    pub string_interner: StringInterner,
    pub pair_manager: PairManager,
    pub source_mapper: SourceMapper,
    pub tracing: bool,
    pub max_stack_size: usize,
    pub keyboard_interrupt_channel: Option<Receiver<()>>,
    has_evaluated_library: bool,
    next_id: u32,
    stack: Vec<SourceRange>,
    stack_traversal_root: GCRootManager<SourceValue>,
}

impl Interpreter {
    pub fn new() -> Self {
        let source_mapper = SourceMapper::default();
        let mut string_interner = StringInterner::default();
        let pair_manager = PairManager::default();
        let mut environment = Environment::default();
        builtins::populate_environment(&mut environment, &mut string_interner);
        Interpreter {
            environment,
            string_interner,
            pair_manager,
            source_mapper,
            tracing: false,
            max_stack_size: DEFAULT_MAX_STACK_SIZE,
            keyboard_interrupt_channel: None,
            next_id: 1,
            stack: vec![],
            stack_traversal_root: GCRootManager::default(),
            has_evaluated_library: false,
        }
    }

    pub fn new_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn expect_number(&mut self, expression: &SourceValue) -> Result<f64, RuntimeError> {
        if let Value::Number(number) = self.eval_expression(&expression)?.0 {
            Ok(number)
        } else {
            Err(RuntimeErrorType::ExpectedNumber.source_mapped(expression.1))
        }
    }

    pub fn print_stats(&self) {
        self.pair_manager.print_stats();
        self.environment.print_stats();
        println!(
            "Objects in call stack: {}",
            self.stack_traversal_root.stats()
        );
        println!("Interned strings: {}", self.string_interner.len());
    }

    fn expect_procedure(&mut self, expression: &SourceValue) -> Result<Procedure, RuntimeError> {
        if let Value::Procedure(procedure) = self.eval_expression(&expression)?.0 {
            Ok(procedure)
        } else {
            Err(RuntimeErrorType::ExpectedProcedure.source_mapped(expression.1))
        }
    }

    fn eval_procedure(
        &mut self,
        procedure: Procedure,
        combination: SourceMapped<&Rc<Vec<SourceValue>>>,
        operands: &[SourceValue],
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
        expression: &SourceValue,
    ) -> Result<Option<TailCallContext>, RuntimeError> {
        match &expression.0 {
            Value::Pair(pair) => {
                // TODO: A lot of this is duplicated from eval_expression, it'd be nice to consolidate
                // somehow.
                let Some(expressions) = pair.try_as_rc_list() else {
                    return Err(RuntimeErrorType::MalformedExpression.source_mapped(expression.1));
                };
                // Unwrap b/c it's from a pair, guaranteed not to be an empty list.
                let operator = expressions.get(0).unwrap();
                let procedure = self.expect_procedure(operator)?;
                let combination = SourceMapped(&expressions, expression.1);
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

    pub fn eval_expression_in_tail_context(&mut self, expression: &SourceValue) -> ProcedureResult {
        if let Some(tail_call_context) = self.try_bind_tail_call_context(expression)? {
            Ok(ProcedureSuccess::TailCall(tail_call_context))
        } else {
            self.lazy_eval_expression(expression)
        }
    }

    fn lazy_eval_expression(&mut self, expression: &SourceValue) -> ProcedureResult {
        match &expression.0 {
            Value::EmptyList | Value::Undefined | Value::Procedure(_) => {
                Err(RuntimeErrorType::MalformedExpression.source_mapped(expression.1))
            }
            Value::Number(number) => Ok(Value::Number(*number).into()),
            Value::Boolean(boolean) => Ok(Value::Boolean(*boolean).into()),
            Value::Symbol(identifier) => {
                if let Some(value) = self.environment.get(identifier) {
                    Ok(value.into())
                } else {
                    Err(RuntimeErrorType::UnboundVariable(identifier.clone())
                        .source_mapped(expression.1))
                }
            }
            Value::Pair(pair) => {
                let Some(expressions) = pair.try_as_rc_list() else {
                    return Err(RuntimeErrorType::MalformedExpression.source_mapped(expression.1));
                };
                // Unwrap b/c it's from a pair, guaranteed not to be an empty list.
                let operator = expressions.get(0).unwrap();
                let procedure = self.expect_procedure(operator)?;
                let combination = SourceMapped(&expressions, expression.1);
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

    pub fn eval_expression(
        &mut self,
        expression: &SourceValue,
    ) -> Result<SourceValue, RuntimeError> {
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
        expressions: &[SourceValue],
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

    pub fn eval_expressions(
        &mut self,
        expressions: &[SourceValue],
    ) -> Result<SourceValue, RuntimeError> {
        let mut last_value: SourceValue = Value::Undefined.into();
        for expression in expressions {
            last_value = self.eval_expression(expression)?;
        }
        Ok(last_value)
    }

    pub fn parse(&mut self, source_id: SourceId) -> Result<Vec<SourceValue>, ParseError> {
        let code = self.source_mapper.get_contents(source_id);
        parse(
            code,
            &mut self.string_interner,
            &mut self.pair_manager,
            Some(source_id),
        )
    }

    pub fn evaluate(&mut self, source_id: SourceId) -> Result<SourceValue, RuntimeError> {
        if !self.has_evaluated_library {
            let library_contents = include_str!("../library/library.sch");
            let library_source_id = self
                .source_mapper
                .add("library.sch".to_string(), library_contents.to_string());
            self.evaluate_source_id(library_source_id)?;
            self.has_evaluated_library = true;
        }
        self.evaluate_source_id(source_id)
    }

    pub fn evaluate_source_id(&mut self, source_id: SourceId) -> Result<SourceValue, RuntimeError> {
        // TODO: The method isn't re-entrant, we should raise an error or
        // something if we detect we're being called in a re-entrant way (or
        // alternatively, make this method re-entrant).
        self.stack.clear();
        self.environment.clear_lexical_scopes();
        match self.parse(source_id) {
            Ok(expressions) => {
                let mut last_value: SourceValue = Value::Undefined.into();
                for expression in self.stack_traversal_root.root_many(expressions) {
                    last_value = self.eval_expression(expression.deref())?;
                }
                Ok(last_value)
            }
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

    pub fn gc(&mut self, debug: bool) -> usize {
        if self.stack.len() > 1 {
            // It would be nice to support this at some point, but right now we can't
            // because we're not pinning temporary objects in the call stack to the GC
            // root--as a result, unexpected things would be considered unreachable and
            // GC'd.
            println!("Cannot currently collect garbage when call stack is non-empty.");
            return 0;
        }
        let mut visitor = Visitor::default();
        visitor.debug = debug;
        self.environment.begin_mark();
        self.pair_manager.begin_mark();
        visitor.traverse(&self.environment);
        visitor.traverse(&self.stack_traversal_root);
        let env_cycles = self.environment.sweep();
        let pair_cycles = self.pair_manager.sweep();
        if visitor.debug {
            println!("Lexical scopes reclaimed: {}", env_cycles);
            println!("Pairs reclaimed: {}", pair_cycles);
        }
        env_cycles + pair_cycles
    }
}
