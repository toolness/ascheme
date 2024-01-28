use std::rc::Rc;

use crate::{
    interpreter::{ProcedureContext, RuntimeError, RuntimeErrorType, Value},
    parser::Expression,
    source_mapped::{SourceMappable, SourceMapped},
    string_interner::InternedString,
};

type CombinationBody = Vec<Expression>;

#[derive(Debug)]
pub struct CompoundProcedure {
    // This isn't technically needed, since the signature is the second element of the definition.
    signature: SourceMapped<Rc<CombinationBody>>,
    definition: SourceMapped<Rc<CombinationBody>>,
}

impl CompoundProcedure {
    pub fn create(
        signature: SourceMapped<Rc<CombinationBody>>,
        definition: SourceMapped<Rc<CombinationBody>>,
    ) -> Result<(InternedString, Self), RuntimeError> {
        let (name, ..) = parse_signature(&signature)?;
        get_body(&definition)?;
        Ok((
            name,
            CompoundProcedure {
                signature,
                definition,
            },
        ))
    }

    pub fn call(&self, mut ctx: ProcedureContext) -> Result<Value, RuntimeError> {
        ctx.interpreter.environment.push();

        let result = self.call_within_local_environment(&mut ctx);

        ctx.interpreter.environment.pop();

        result
    }

    fn call_within_local_environment(
        &self,
        ctx: &mut ProcedureContext,
    ) -> Result<Value, RuntimeError> {
        // We're unwrapping these because we already validated them upon construction.
        let (.., arg_bindings) = parse_signature(&self.signature).unwrap();
        let body = get_body(&self.definition).unwrap();

        if ctx.operands.len() != arg_bindings.len() {
            return Err(RuntimeErrorType::WrongNumberOfArguments.source_mapped(ctx.combination.1));
        }

        for (expr, name) in ctx.operands.iter().zip(arg_bindings) {
            let value = ctx.interpreter.eval_expression(expr)?;
            ctx.interpreter.environment.set(name, value);
        }
        ctx.interpreter.eval_expressions(body)
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
) -> Result<(InternedString, Vec<InternedString>), RuntimeError> {
    let Some(first) = signature.0.get(0) else {
        return Err(RuntimeErrorType::MalformedSpecialForm.source_mapped(signature.1));
    };
    let name = first.expect_identifier()?;
    let mut arg_bindings: Vec<InternedString> = vec![];
    for arg_name in &signature.0[1..] {
        arg_bindings.push(arg_name.expect_identifier()?);
    }
    Ok((name, arg_bindings))
}

// TODO: This should really be done for us by a generic trait.
impl Clone for CompoundProcedure {
    fn clone(&self) -> Self {
        Self {
            signature: SourceMapped(self.signature.0.clone(), self.signature.1.clone()),
            definition: SourceMapped(self.definition.0.clone(), self.definition.1.clone()),
        }
    }
}

impl PartialEq for CompoundProcedure {
    /// Just compare pointers of the underlying value.
    fn eq(&self, other: &Self) -> bool {
        &*self.definition.0 as *const CombinationBody
            == &*other.definition.0 as *const CombinationBody
    }
}
