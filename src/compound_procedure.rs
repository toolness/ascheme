use std::{collections::HashSet, rc::Rc};

use crate::{
    environment::{CapturedLexicalScope, Environment},
    gc::{Traverser, Visitor},
    interpreter::{Interpreter, ProcedureContext, ProcedureResult, RuntimeError, RuntimeErrorType},
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

    fn check_arity(&self, args_len: usize, range: SourceRange) -> Result<(), RuntimeError> {
        let is_valid = match self {
            Signature::FixedArgs(args) => args_len == args.len(),
            Signature::MinArgs(args, _) => args_len >= args.len(),
            Signature::AnyArgs(_) => true,
        };
        if is_valid {
            Ok(())
        } else {
            Err(RuntimeErrorType::WrongNumberOfArguments.source_mapped(range))
        }
    }

    fn bind_args(&self, operands: Vec<SourceValue>, environment: &mut Environment) {
        match self {
            Signature::FixedArgs(arg_names) => {
                for (name, value) in arg_names.iter().zip(operands) {
                    environment.define(name.0.clone(), value);
                }
            }
            Signature::MinArgs(_, _) => todo!("IMPLEMENT MIN ARGS BINDING"),
            Signature::AnyArgs(_) => todo!("IMPLEMENT ANY ARGS BINDING"),
        }
    }
}

type CombinationBody = Vec<SourceValue>;

#[derive(Debug, Clone)]
pub struct CompoundProcedure {
    pub name: Option<InternedString>,
    id: u32,
    signature: Rc<Signature>,
    definition: SourceMapped<Rc<CombinationBody>>,
    captured_lexical_scope: CapturedLexicalScope,
}

impl CompoundProcedure {
    pub fn create(
        id: u32,
        signature: Signature,
        definition: SourceMapped<Rc<CombinationBody>>,
        captured_lexical_scope: CapturedLexicalScope,
    ) -> Result<Self, RuntimeError> {
        get_body(&definition)?;
        Ok(CompoundProcedure {
            name: None,
            id,
            signature: Rc::new(signature),
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
        self.signature
            .check_arity(ctx.operands.len(), ctx.combination.1)?;
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
}

impl Traverser for CompoundProcedure {
    fn traverse(&self, visitor: &Visitor) {
        visitor.traverse(&self.definition);
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
            self.procedure.definition.1,
        );

        let body = self.procedure.body();
        self.procedure
            .signature
            .bind_args(self.operands, &mut interpreter.environment);

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
