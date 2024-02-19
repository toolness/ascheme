use crate::{
    interpreter::{
        Interpreter, Procedure, ProcedureContext, ProcedureResult, RuntimeError, RuntimeErrorType,
    },
    source_mapped::SourceMappable,
    value::{SourceValue, Value},
};

pub fn is_eq(
    interpreter: &mut Interpreter,
    a: &SourceValue,
    b: &SourceValue,
) -> Result<bool, RuntimeError> {
    let a = interpreter.eval_expression(&a)?;
    let b = interpreter.eval_expression(&b)?;

    Ok(match a.0 {
        Value::Undefined => matches!(b.0, Value::Undefined),
        Value::EmptyList => matches!(b.0, Value::EmptyList),
        Value::Number(a) => match b.0 {
            Value::Number(b) => a == b,
            _ => false,
        },
        Value::Symbol(a) => match &b.0 {
            Value::Symbol(b) => &a == b,
            _ => false,
        },
        Value::Boolean(a) => match b.0 {
            Value::Boolean(b) => a == b,
            _ => false,
        },
        Value::String(a) => match &b.0 {
            Value::String(b) => a.points_at_same_memory_as(b),
            _ => false,
        },
        Value::Procedure(Procedure::Builtin(a, _)) => match &b.0 {
            Value::Procedure(Procedure::Builtin(b, _)) => a == *b,
            _ => false,
        },
        Value::Procedure(Procedure::Compound(a)) => match &b.0 {
            Value::Procedure(Procedure::Compound(b)) => a.id() == b.id(),
            _ => false,
        },
        Value::Pair(a) => match &b.0 {
            Value::Pair(b) => a.points_at_same_memory_as(b),
            _ => false,
        },
    })
}

pub fn eq(mut ctx: ProcedureContext) -> ProcedureResult {
    if ctx.operands.len() < 2 {
        return Err(RuntimeErrorType::WrongNumberOfArguments.source_mapped(ctx.combination.1));
    }

    Ok(is_eq(&mut ctx.interpreter, &ctx.operands[0], &ctx.operands[1])?.into())
}
