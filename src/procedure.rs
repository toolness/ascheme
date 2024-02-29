use crate::{
    bound_procedure::BoundProcedure,
    builtin_procedure::BuiltinProcedure,
    compound_procedure::CompoundProcedure,
    interpreter::{Interpreter, RuntimeError, RuntimeErrorType},
    source_mapped::{SourceMappable, SourceRange},
    string_interner::InternedString,
    value::SourceValue,
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

    fn check_arity(&self, operands_len: usize, range: SourceRange) -> Result<(), RuntimeError> {
        if !self.is_valid_arity(operands_len) {
            Err(RuntimeErrorType::WrongNumberOfArguments.source_mapped(range))
        } else {
            Ok(())
        }
    }

    pub fn bind(
        self,
        range: SourceRange,
        operands: &[SourceValue],
    ) -> Result<BoundProcedure, RuntimeError> {
        self.check_arity(operands.len(), range)?;
        Ok(BoundProcedure {
            procedure: self,
            operands: Vec::from(operands),
            range,
        })
    }

    pub fn eval_and_bind(
        self,
        interpreter: &mut Interpreter,
        range: SourceRange,
        operands: &[SourceValue],
    ) -> Result<BoundProcedure, RuntimeError> {
        self.check_arity(operands.len(), range)?;
        let mut evaluated_operands = Vec::with_capacity(operands.len());
        for expr in operands.iter() {
            let value = interpreter.eval_expression(expr)?;
            evaluated_operands.push(value);
        }
        Ok(BoundProcedure {
            procedure: self,
            operands: evaluated_operands,
            range,
        })
    }
}
