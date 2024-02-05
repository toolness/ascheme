use std::rc::Rc;

use crate::{
    environment::CapturedLexicalScope,
    interpreter::{
        Interpreter, ProcedureContext, ProcedureResult, RuntimeError, RuntimeErrorType, Value,
    },
    parser::Expression,
    source_mapped::{SourceMappable, SourceMapped},
    string_interner::InternedString,
};

type CombinationBody = Vec<Expression>;

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

    fn body(&self) -> &[Expression] {
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

pub struct BoundProcedure {
    procedure: CompoundProcedure,
    operands: Vec<Value>,
}

impl BoundProcedure {
    pub fn call(self, interpreter: &mut Interpreter) -> ProcedureResult {
        interpreter.environment.push(
            self.procedure.captured_lexical_scope.clone(),
            self.procedure.signature.1,
        );

        let body = self.procedure.body();
        let arg_bindings = self.procedure.arg_bindings();

        for (name, value) in arg_bindings.into_iter().zip(self.operands) {
            interpreter.environment.set(name, value);
        }

        let result = interpreter.eval_expressions_in_tail_context(body)?;

        // Note that the environment won't have been popped if an error occured above--this is
        // so we can examine it afterwards, if needed. It's up to the caller to clean things
        // up after an error.
        interpreter.environment.pop();

        Ok(result)
    }
}

fn get_body(definition: &SourceMapped<Rc<CombinationBody>>) -> Result<&[Expression], RuntimeError> {
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
        arg_bindings.push(arg_name.expect_identifier()?);
    }
    Ok(arg_bindings)
}

impl PartialEq for CompoundProcedure {
    /// Just compare pointers of the underlying value.
    fn eq(&self, other: &Self) -> bool {
        &*self.definition.0 as *const CombinationBody
            == &*other.definition.0 as *const CombinationBody
    }
}
