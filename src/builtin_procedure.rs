use crate::{
    interpreter::{CallableResult, Interpreter},
    source_mapped::{SourceMappable, SourceRange},
    string_interner::InternedString,
    value::{SourceValue, Value},
};

pub struct BuiltinProcedureContext<'a> {
    pub interpreter: &'a mut Interpreter,
    pub range: SourceRange,
}

impl<'a> BuiltinProcedureContext<'a> {
    pub fn undefined(&self) -> CallableResult {
        Ok(Value::Undefined.source_mapped(self.range).into())
    }
}

#[derive(Debug, Clone)]
pub struct BuiltinProcedure {
    pub func: BuiltinProcedureFn,
    pub name: InternedString,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BuiltinProcedureFn {
    Nullary(fn(BuiltinProcedureContext) -> CallableResult),
    Unary(fn(BuiltinProcedureContext, &SourceValue) -> CallableResult),
    Binary(fn(BuiltinProcedureContext, &SourceValue, &SourceValue) -> CallableResult),
    NullaryVariadic(fn(BuiltinProcedureContext, &[SourceValue]) -> CallableResult),
    UnaryVariadic(fn(BuiltinProcedureContext, &SourceValue, &[SourceValue]) -> CallableResult),
}

impl BuiltinProcedure {
    pub fn is_valid_arity(&self, operands_len: usize) -> bool {
        match self.func {
            BuiltinProcedureFn::Nullary(_) => operands_len == 0,
            BuiltinProcedureFn::Unary(_) => operands_len == 1,
            BuiltinProcedureFn::Binary(_) => operands_len == 2,
            BuiltinProcedureFn::NullaryVariadic(_) => true,
            BuiltinProcedureFn::UnaryVariadic(_) => operands_len >= 1,
        }
    }

    pub fn call(&self, ctx: BuiltinProcedureContext, operands: Vec<SourceValue>) -> CallableResult {
        match self.func {
            BuiltinProcedureFn::Nullary(func) => (func)(ctx),
            BuiltinProcedureFn::Unary(func) => (func)(ctx, &operands[0]),
            BuiltinProcedureFn::Binary(func) => (func)(ctx, &operands[0], &operands[1]),
            BuiltinProcedureFn::NullaryVariadic(func) => (func)(ctx, &operands[..]),
            BuiltinProcedureFn::UnaryVariadic(func) => (func)(ctx, &operands[0], &operands[1..]),
        }
    }
}
