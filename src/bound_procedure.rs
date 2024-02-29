use crate::{
    builtin_procedure::BuiltinProcedureContext,
    interpreter::{CallableResult, Interpreter, Procedure},
    source_mapped::SourceRange,
    string_interner::InternedString,
    value::SourceValue,
};

pub struct BoundProcedure {
    pub procedure: Procedure,
    pub operands: Vec<SourceValue>,
    pub range: SourceRange,
}

impl BoundProcedure {
    pub fn name(&self) -> Option<&InternedString> {
        self.procedure.name()
    }

    pub fn call(self, interpreter: &mut Interpreter) -> CallableResult {
        match self.procedure {
            Procedure::Compound(compound) => compound.call(interpreter, self.operands),
            Procedure::Builtin(builtin) => {
                let ctx = BuiltinProcedureContext {
                    interpreter,
                    range: self.range,
                };
                builtin.call(ctx, self.operands)
            }
        }
    }
}
