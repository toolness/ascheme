use std::{collections::HashSet, rc::Rc};

use crate::{
    environment::CapturedLexicalScope,
    gc::{Traverser, Visitor},
    interpreter::{CallableResult, Interpreter, RuntimeError, RuntimeErrorType},
    pair::PairVisitedSet,
    source_mapped::{SourceMappable, SourceMapped, SourceRange},
    string_interner::InternedString,
    value::{SourceValue, Value},
};

#[derive(Debug)]
pub enum Signature {
    FixedArgs(Vec<SourceMapped<InternedString>>),
    MinArgs(
        Vec<SourceMapped<InternedString>>,
        SourceMapped<InternedString>,
    ),
    AnyArgs(SourceMapped<InternedString>),
}

impl Signature {
    pub fn parse(value: SourceValue) -> Result<Self, RuntimeError> {
        match value.0 {
            Value::EmptyList => Ok(Signature::FixedArgs(vec![])),
            Value::Symbol(name) => Ok(Signature::AnyArgs(name.source_mapped(value.1))),
            Value::Pair(mut pair) => {
                let mut visited = PairVisitedSet::default();
                let mut args: Vec<SourceMapped<InternedString>> = vec![];
                let mut args_set: HashSet<InternedString> = HashSet::default();
                loop {
                    visited.add(&pair);
                    let car = pair.car();
                    let name = car.expect_identifier()?;
                    if !args_set.insert(name.clone()) {
                        return Err(RuntimeErrorType::DuplicateParameter.source_mapped(car.1));
                    }
                    args.push(name.source_mapped(car.1));
                    let cdr = pair.cdr();
                    match cdr.0 {
                        Value::EmptyList => return Ok(Signature::FixedArgs(args)),
                        Value::Symbol(name) => {
                            if args_set.contains(&name) {
                                return Err(
                                    RuntimeErrorType::DuplicateParameter.source_mapped(cdr.1)
                                );
                            }
                            return Ok(Signature::MinArgs(args, name.source_mapped(cdr.1)));
                        }
                        Value::Pair(next) => {
                            if visited.contains(&next) {
                                return Err(
                                    RuntimeErrorType::MalformedSpecialForm.source_mapped(cdr.1)
                                );
                            }
                            pair = next;
                        }
                        _ => {
                            return Err(RuntimeErrorType::MalformedSpecialForm.source_mapped(cdr.1))
                        }
                    }
                }
            }
            _ => Err(RuntimeErrorType::MalformedSpecialForm.source_mapped(value.1)),
        }
    }

    pub fn is_valid_arity(&self, args_len: usize) -> bool {
        match self {
            Signature::FixedArgs(args) => args_len == args.len(),
            Signature::MinArgs(args, _) => args_len >= args.len(),
            Signature::AnyArgs(_) => true,
        }
    }

    fn bind_args(&self, mut operands: Vec<SourceValue>, interpreter: &mut Interpreter) {
        match self {
            Signature::FixedArgs(arg_names) => {
                for (name, value) in arg_names.iter().zip(operands) {
                    interpreter.environment.define(name.0.clone(), value);
                }
            }
            Signature::MinArgs(required_arg_names, rest_arg_name) => {
                let rest_operands = operands.split_off(required_arg_names.len());
                for (name, value) in required_arg_names.iter().zip(operands) {
                    interpreter.environment.define(name.0.clone(), value);
                }
                interpreter.environment.define(
                    rest_arg_name.0.clone(),
                    interpreter
                        .pair_manager
                        .vec_to_list(rest_operands)
                        .source_mapped(rest_arg_name.1),
                );
            }
            Signature::AnyArgs(arg_name) => {
                interpreter.environment.define(
                    arg_name.0.clone(),
                    interpreter
                        .pair_manager
                        .vec_to_list(operands)
                        .source_mapped(arg_name.1),
                );
            }
        }
    }
}

#[derive(Debug)]
pub struct Body(SourceMapped<Vec<SourceValue>>);

impl Body {
    pub fn try_new(body: &[SourceValue], range: SourceRange) -> Result<Self, RuntimeError> {
        if body.is_empty() {
            Err(RuntimeErrorType::MalformedSpecialForm.source_mapped(range))
        } else {
            Ok(Body(Vec::from(body).source_mapped(range)))
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompoundProcedure {
    pub name: Option<InternedString>,
    id: u32,
    pub signature: Rc<Signature>,
    body: Rc<Body>,
    captured_lexical_scope: CapturedLexicalScope,
}

impl CompoundProcedure {
    pub fn create(
        id: u32,
        signature: Signature,
        body: Body,
        captured_lexical_scope: CapturedLexicalScope,
    ) -> Self {
        CompoundProcedure {
            name: None,
            id,
            signature: Rc::new(signature),
            body: Rc::new(body),
            captured_lexical_scope,
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn call(
        &self,
        interpreter: &mut Interpreter,
        operands: Vec<SourceValue>,
    ) -> CallableResult {
        interpreter
            .environment
            .push(self.captured_lexical_scope.clone(), self.body.0 .1);

        let body = &self.body.0 .0;
        self.signature.bind_args(operands, interpreter);

        let result = interpreter.eval_expressions_in_tail_context(body)?;

        // Note that the environment won't have been popped if an error occured above--this is
        // so we can examine it afterwards, if needed. It's up to the caller to clean things
        // up after an error.
        interpreter.environment.pop();

        Ok(result)
    }
}

impl Traverser for CompoundProcedure {
    fn traverse(&self, visitor: &Visitor) {
        visitor.traverse(&self.body.0);
        visitor.traverse(&self.captured_lexical_scope);
    }
}
