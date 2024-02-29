use std::{ops::Deref, sync::mpsc::Receiver};

use crate::{
    bound_procedure::BoundProcedure,
    builtins::{self, add_library_source},
    environment::Environment,
    gc::Visitor,
    gc_rooted::GCRootManager,
    pair::PairManager,
    parser::{parse, ParseError, ParseErrorType},
    procedure::Procedure,
    source_mapped::{SourceMappable, SourceMapped, SourceRange},
    source_mapper::{SourceId, SourceMapper},
    special_form::{SpecialForm, SpecialFormContext},
    stdio_printer::StdioPrinter,
    string_interner::{InternedString, StringInterner},
    tracked_stats::TrackedStats,
    value::{SourceValue, Value},
};

const DEFAULT_MAX_STACK_SIZE: usize = 128;

#[derive(Debug, PartialEq)]
pub enum RuntimeErrorType {
    Parse(ParseErrorType),
    UnboundVariable(InternedString),
    MalformedExpression,
    MalformedSpecialForm,
    MalformedBindingList,
    ExpectedNumber,
    ExpectedCallable,
    ExpectedProcedure,
    ExpectedIdentifier,
    ExpectedPair,
    ExpectedList,
    WrongNumberOfArguments,
    DuplicateParameter,
    DuplicateVariableInBindings,
    StackOverflow,
    KeyboardInterrupt,
    DivisionByZero,
    AssertionFailure,
}

pub type RuntimeError = SourceMapped<RuntimeErrorType>;

impl From<ParseError> for RuntimeError {
    fn from(value: ParseError) -> Self {
        RuntimeErrorType::Parse(value.0).source_mapped(value.1)
    }
}

impl<T: Into<SourceValue>> From<T> for CallableSuccess {
    fn from(value: T) -> Self {
        CallableSuccess::Value(value.into())
    }
}

#[derive(Debug, Clone)]
pub enum Callable {
    SpecialForm(SpecialForm),
    Procedure(Procedure),
}

pub type CallableResult = Result<CallableSuccess, RuntimeError>;

pub struct TailCallContext {
    pub bound_procedure: BoundProcedure,
}

pub enum CallableSuccess {
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
    pub printer: StdioPrinter,
    pub failed_tests: usize,
    tracked_stats: Option<TrackedStats>,
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
            tracked_stats: None,
            printer: StdioPrinter::new(),
            failed_tests: 0,
        }
    }

    pub fn new_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn print_stats(&self) {
        self.printer
            .println(self.pair_manager.get_stats_as_string());
        self.printer.println(self.environment.get_stats_as_string());
        self.printer.println(format!(
            "Objects in call stack: {}",
            self.stack_traversal_root.stats()
        ));
        self.printer
            .println(format!("Interned strings: {}", self.string_interner.len()));
    }

    pub fn show_err_and_traceback(&self, err: RuntimeError) {
        self.printer.eprintln(format!(
            "Error: {:?} in {}",
            err.0,
            self.source_mapper.trace(&err.1).join("\n")
        ));
        self.printer.eprintln(self.traceback());
    }

    fn expect_callable(&mut self, expression: &SourceValue) -> Result<Callable, RuntimeError> {
        if let Value::Callable(callable) = self.eval_expression(&expression)?.0 {
            Ok(callable)
        } else {
            Err(RuntimeErrorType::ExpectedCallable.source_mapped(expression.1))
        }
    }

    fn eval_callable(
        &mut self,
        callable: Callable,
        operands: &[SourceValue],
        operator_source_range: SourceRange,
        combination_source_range: SourceRange,
    ) -> CallableResult {
        match callable {
            Callable::SpecialForm(special_form) => {
                let ctx = SpecialFormContext {
                    interpreter: self,
                    range: combination_source_range,
                    operands,
                };
                (special_form.func)(ctx)
            }
            Callable::Procedure(procedure) => {
                if self.stack.len() >= self.max_stack_size {
                    return Err(
                        RuntimeErrorType::StackOverflow.source_mapped(combination_source_range)
                    );
                }
                self.stack.push(operator_source_range);
                if let Some(ref mut stats) = &mut self.tracked_stats {
                    stats.update_call_stack_depth(self.stack.len());
                    stats.track_call(procedure.name());
                }
                let bound = procedure.eval_and_bind(self, combination_source_range, operands)?;
                let result = bound.call(self)?;
                // Note that the stack won't unwind if an error occured above--this is so we can get a stack trace
                // afterwards. It's up to the caller to clean things up after an error.
                self.stack.pop();
                Ok(result)
            }
        }
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
                let callable = self.expect_callable(operator)?;
                let combination = SourceMapped(&expressions, expression.1);
                let operands = &expressions[1..];
                match callable {
                    Callable::Procedure(procedure) => Ok(Some(TailCallContext {
                        bound_procedure: procedure.eval_and_bind(self, combination.1, operands)?,
                    })),
                    Callable::SpecialForm(_) => Ok(None),
                }
            }
            _ => Ok(None),
        }
    }

    pub fn eval_expression_in_tail_context(&mut self, expression: &SourceValue) -> CallableResult {
        if let Some(tail_call_context) = self.try_bind_tail_call_context(expression)? {
            Ok(CallableSuccess::TailCall(tail_call_context))
        } else {
            self.lazy_eval_expression(expression)
        }
    }

    fn lazy_eval_expression(&mut self, expression: &SourceValue) -> CallableResult {
        match &expression.0 {
            Value::EmptyList | Value::Callable(_) => {
                Err(RuntimeErrorType::MalformedExpression.source_mapped(expression.1))
            }
            Value::Undefined => Ok(Value::Undefined.into()),
            Value::Number(number) => Ok(Value::Number(*number).into()),
            Value::Boolean(boolean) => Ok(Value::Boolean(*boolean).into()),
            Value::String(string) => Ok(Value::String(string.clone()).into()),
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
                let callable = self.expect_callable(operator)?;
                let combination = SourceMapped(&expressions, expression.1);
                let operands = &expressions[1..];
                if self.tracing {
                    self.printer.println(format!(
                        "Evaluating callable {}",
                        self.source_mapper.trace(&combination.1).join("\n")
                    ));
                }
                self.eval_callable(callable, operands, operator.1, combination.1)
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
                CallableSuccess::Value(value) => return Ok(value),
                CallableSuccess::TailCall(tail_call_context) => {
                    if let Some(ref mut stats) = &mut self.tracked_stats {
                        stats.track_tail_call(tail_call_context.bound_procedure.name())
                    }
                    if self.tracing {
                        self.printer.println(format!(
                            "Evaluating tail call {}",
                            self.source_mapper
                                .trace(&tail_call_context.bound_procedure.range)
                                .join("\n")
                        ));
                    }
                    result = tail_call_context.bound_procedure.call(self)?;
                }
            }
        }
    }

    pub fn eval_expressions_in_tail_context(
        &mut self,
        expressions: &[SourceValue],
    ) -> Result<CallableSuccess, RuntimeError> {
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
            let library_source_id = add_library_source(&mut self.source_mapper);
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
            self.printer
                .println("Cannot currently collect garbage when call stack is non-empty.");
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
            self.printer.println(format!(
                "Lexical scopes reclaimed: {env_cycles}\nPairs reclaimed: {pair_cycles}",
            ));
        }
        env_cycles + pair_cycles
    }

    pub fn start_tracking_stats(&mut self) {
        self.tracked_stats = Some(TrackedStats::default())
    }

    pub fn take_tracked_stats(&mut self) -> Option<TrackedStats> {
        self.tracked_stats.take()
    }
}

#[cfg(test)]
mod tests {
    use crate::test_util::test_eval_success;

    #[test]
    fn trivial_expressions_work() {
        test_eval_success("5", "5");
    }

    #[test]
    fn dot_works() {
        test_eval_success("(quote (1 . ()))", "(1)");
        test_eval_success("(quote (1 . (2 . (3 . ()))))", "(1 2 3)");
        test_eval_success("(quote (1 . 2))", "(1 . 2)");
        test_eval_success("(quote (1 2 . 3))", "(1 2 . 3)");
    }

    #[test]
    fn booleans_work() {
        test_eval_success("#t", "#t");
        test_eval_success("#f", "#f");
    }

    #[test]
    fn undefined_works() {
        test_eval_success("#!void", "");
    }

    #[test]
    fn cyclic_lists_work() {
        // TODO: Eventually we should implement proper display of cyclic lists, at which point
        // the expected values will need to change.
        test_eval_success("(define x '(1 . 2)) (set-cdr! x x) x", "<CYCLIC LIST>");
        test_eval_success(
            "(define y '(1)) (define x '(1)) (set-car! y x) (set-car! x y) x",
            "<CYCLIC LIST>",
        );
    }

    #[test]
    fn gc_finds_cycles() {
        // These print 0 because there aren't any objects trapped in cycles--regular ref-counting
        // will clean up the data.
        test_eval_success("(gc)", "0");
        test_eval_success("(define (x n) (+ n 1)) (gc)", "0");
        test_eval_success("(define (x n) (+ n 1)) (define x 0) (gc)", "0");

        // This prints 1 because an object is caught in a cycle.
        test_eval_success(
            "(define x (quote (1 . 2))) (set-cdr! x x) (define x 0) (gc)",
            "1",
        );
    }

    #[test]
    fn gc_does_not_collect_objects_yet_to_be_evaluated() {
        test_eval_success("(define (x) 1) (gc) (x)", "1");
    }

    #[test]
    fn strings_work() {
        test_eval_success(r#""blarg""#, r#""blarg""#);
        test_eval_success(r#""bl\narg""#, r#""bl\narg""#);
        test_eval_success(r#""bl\"arg""#, r#""bl\"arg""#);
        test_eval_success(r#""bl\\arg""#, r#""bl\\arg""#);
    }

    #[test]
    fn undefined_stringifies() {
        test_eval_success(
            "
        (define y 1)
        (define x '(1))
        (set-car! x (set! y 2))
        x
        ",
            "(#!void)",
        )
    }
}
