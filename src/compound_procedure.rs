use std::rc::Rc;

use crate::{
    environment::CapturedLexicalScope,
    gc::{Traverser, Visitor},
    interpreter::{Interpreter, ProcedureContext, ProcedureResult, RuntimeError, RuntimeErrorType},
    pair::PairVisitedSet,
    source_mapped::{SourceMappable, SourceMapped},
    string_interner::InternedString,
    value::{SourceValue, Value},
};

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
            Value::Symbol(name) => Ok(Signature::AnyArgs(name.source_mapped(value.1))),
            Value::Pair(mut pair) => {
                let mut visited = PairVisitedSet::default();
                let mut args: Vec<SourceMapped<InternedString>> = vec![];
                loop {
                    visited.add(&pair);
                    let car = pair.car();
                    args.push(car.expect_identifier()?.source_mapped(car.1));
                    let cdr = pair.cdr();
                    match cdr.0 {
                        Value::EmptyList => return Ok(Signature::FixedArgs(args)),
                        Value::Symbol(name) => {
                            return Ok(Signature::MinArgs(args, name.source_mapped(cdr.1)))
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
}

type CombinationBody = Vec<SourceValue>;

#[derive(Debug, Clone)]
pub struct CompoundProcedure {
    pub name: Option<InternedString>,
    id: u32,
    // This isn't technically needed, since the signature is the second element of the definition.
    signature: SourceMapped<Rc<CombinationBody>>,
    signature_first_arg_index: usize,
    definition: SourceMapped<Rc<CombinationBody>>,
    captured_lexical_scope: CapturedLexicalScope,
}

impl CompoundProcedure {
    pub fn create(
        id: u32,
        signature: SourceMapped<Rc<CombinationBody>>,
        signature_first_arg_index: usize,
        definition: SourceMapped<Rc<CombinationBody>>,
        captured_lexical_scope: CapturedLexicalScope,
    ) -> Result<Self, RuntimeError> {
        parse_signature(&signature, signature_first_arg_index)?;
        get_body(&definition)?;
        Ok(CompoundProcedure {
            name: None,
            id,
            signature,
            signature_first_arg_index,
            definition,
            captured_lexical_scope,
        })
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn call(self, mut ctx: ProcedureContext) -> ProcedureResult {
        let bound_procedure = self.bind(&mut ctx)?;
        bound_procedure.call(&mut ctx.interpreter)
    }

    pub fn bind(self, ctx: &mut ProcedureContext) -> Result<BoundProcedure, RuntimeError> {
        if ctx.operands.len() != self.arity() {
            return Err(RuntimeErrorType::WrongNumberOfArguments.source_mapped(ctx.combination.1));
        }
        let mut operands = Vec::with_capacity(ctx.operands.len());
        for expr in ctx.operands.iter() {
            let value = ctx.interpreter.eval_expression(expr)?;
            operands.push(value);
        }
        Ok(BoundProcedure {
            procedure: self,
            operands,
        })
    }

    fn body(&self) -> &[SourceValue] {
        // We're unwrapping these because we already validated them upon construction.
        get_body(&self.definition).unwrap()
    }

    fn arg_bindings(&self) -> Vec<InternedString> {
        // We're unwrapping these because we already validated them upon construction.
        parse_signature(&self.signature, self.signature_first_arg_index).unwrap()
    }

    fn arity(&self) -> usize {
        self.signature.0.len() - self.signature_first_arg_index
    }
}

impl Traverser for CompoundProcedure {
    fn traverse(&self, visitor: &Visitor) {
        visitor.traverse(&self.definition);
        visitor.traverse(&self.signature);
        visitor.traverse(&self.captured_lexical_scope);
    }
}

pub struct BoundProcedure {
    procedure: CompoundProcedure,
    operands: Vec<SourceValue>,
}

impl BoundProcedure {
    pub fn name(&self) -> Option<&InternedString> {
        self.procedure.name.as_ref()
    }

    pub fn call(self, interpreter: &mut Interpreter) -> ProcedureResult {
        interpreter.environment.push(
            self.procedure.captured_lexical_scope.clone(),
            self.procedure.signature.1,
        );

        let body = self.procedure.body();
        let arg_bindings = self.procedure.arg_bindings();

        for (name, value) in arg_bindings.into_iter().zip(self.operands) {
            interpreter.environment.define(name, value);
        }

        let result = interpreter.eval_expressions_in_tail_context(body)?;

        // Note that the environment won't have been popped if an error occured above--this is
        // so we can examine it afterwards, if needed. It's up to the caller to clean things
        // up after an error.
        interpreter.environment.pop();

        Ok(result)
    }
}

fn get_body(
    definition: &SourceMapped<Rc<CombinationBody>>,
) -> Result<&[SourceValue], RuntimeError> {
    let body = &definition.0[2..];
    if body.is_empty() {
        Err(RuntimeErrorType::MalformedSpecialForm.source_mapped(definition.1))
    } else {
        Ok(body)
    }
}

fn parse_signature(
    signature: &SourceMapped<Rc<CombinationBody>>,
    first_arg_index: usize,
) -> Result<Vec<InternedString>, RuntimeError> {
    let mut arg_bindings: Vec<InternedString> = vec![];
    for arg_name in &signature.0[first_arg_index..] {
        let id = arg_name.expect_identifier()?;
        if arg_bindings.contains(&id) {
            return Err(RuntimeErrorType::DuplicateParameter.source_mapped(arg_name.1));
        }
        arg_bindings.push(id);
    }
    Ok(arg_bindings)
}
