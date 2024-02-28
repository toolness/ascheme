use crate::{
    interpreter::{CallableResult, Interpreter, RuntimeError, RuntimeErrorType},
    source_mapped::{SourceMappable, SourceRange},
    string_interner::InternedString,
    value::{SourceValue, Value},
};

/// Encapsulates all the details of a special
/// form invocation required for evaluation.
///
/// This structure doesn't actually evaluate its operands.
pub struct SpecialFormContext<'a> {
    pub interpreter: &'a mut Interpreter,
    pub range: SourceRange,
    pub operands: &'a [SourceValue],
}

impl<'a> SpecialFormContext<'a> {
    pub fn ensure_operands_len(&self, len: usize) -> Result<(), RuntimeError> {
        if self.operands.len() != len {
            Err(RuntimeErrorType::WrongNumberOfArguments.source_mapped(self.range))
        } else {
            Ok(())
        }
    }

    pub fn eval_unary(&mut self) -> Result<SourceValue, RuntimeError> {
        self.ensure_operands_len(1)?;
        Ok(self.interpreter.eval_expression(&self.operands[0])?)
    }

    pub fn undefined(&self) -> CallableResult {
        Ok(Value::Undefined.source_mapped(self.range).into())
    }
}

#[derive(Debug, Clone)]
pub struct SpecialForm {
    pub func: SpecialFormFn,
    pub name: InternedString,
}

pub type SpecialFormFn = fn(SpecialFormContext) -> CallableResult;
