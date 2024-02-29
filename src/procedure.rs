use crate::{
    builtin_procedure::BuiltinProcedure, compound_procedure::CompoundProcedure,
    string_interner::InternedString,
};

#[derive(Debug, Clone)]
pub enum Procedure {
    Compound(CompoundProcedure),
    Builtin(BuiltinProcedure),
}

impl Procedure {
    pub fn name(&self) -> Option<&InternedString> {
        match self {
            Procedure::Builtin(builtin) => Some(&builtin.name),
            Procedure::Compound(compound) => compound.name.as_ref(),
        }
    }

    pub fn is_valid_arity(&self, operands_len: usize) -> bool {
        match self {
            Procedure::Compound(compound) => compound.signature.is_valid_arity(operands_len),
            Procedure::Builtin(builtin) => builtin.is_valid_arity(operands_len),
        }
    }
}
