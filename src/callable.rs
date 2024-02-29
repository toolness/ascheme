use crate::{
    bound_procedure::BoundProcedure, interpreter::RuntimeError, procedure::Procedure,
    special_form::SpecialForm, value::SourceValue,
};

impl<T: Into<SourceValue>> From<T> for CallableSuccess {
    fn from(value: T) -> Self {
        CallableSuccess::Value(value.into())
    }
}

#[derive(Debug, Clone)]
pub enum Callable {
    SpecialForm(SpecialForm),
    Procedure(Procedure),
}

pub type CallableResult = Result<CallableSuccess, RuntimeError>;

pub struct TailCallContext {
    pub bound_procedure: BoundProcedure,
}

pub enum CallableSuccess {
    Value(SourceValue),
    TailCall(TailCallContext),
}
