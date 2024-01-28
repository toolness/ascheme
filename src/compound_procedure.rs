use std::rc::Rc;

use crate::{
    interpreter::{RuntimeError, RuntimeErrorType},
    parser::Expression,
    source_mapped::{SourceMappable, SourceMapped},
    string_interner::InternedString,
};

type Definition = Vec<Expression>;

// TODO: This could just be a tuple struct.
#[derive(Debug)]
pub struct CompoundProcedure {
    definition: SourceMapped<Rc<Definition>>,
}

impl CompoundProcedure {
    pub fn create(
        signature: SourceMapped<&Vec<Expression>>,
        definition: SourceMapped<Rc<Definition>>,
    ) -> Result<(InternedString, Self), RuntimeError> {
        let (name, ..) = parse_definition(signature)?;
        Ok((name, CompoundProcedure { definition }))
    }
}

fn parse_definition(
    signature: SourceMapped<&Vec<Expression>>,
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
            definition: SourceMapped(self.definition.0.clone(), self.definition.1.clone()),
        }
    }
}

impl PartialEq for CompoundProcedure {
    /// Just compare pointers of the underlying value.
    fn eq(&self, other: &Self) -> bool {
        &*self.definition.0 as *const Definition == &*other.definition.0 as *const Definition
    }
}
