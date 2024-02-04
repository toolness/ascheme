use std::f64::INFINITY;

use crate::{
    compound_procedure::CompoundProcedure,
    environment::Environment,
    interpreter::{
        Procedure, ProcedureContext, ProcedureFn, ProcedureResult, RuntimeError, RuntimeErrorType,
        Value,
    },
    parser::ExpressionValue,
    source_mapped::{SourceMappable, SourceMapped},
    string_interner::StringInterner,
};

pub fn populate_environment(environment: &mut Environment, interner: &mut StringInterner) {
    for (name, builtin) in get_builtins() {
        let interned_name = interner.intern(name);
        environment.set(
            interned_name.clone(),
            Value::Procedure(Procedure::Builtin(builtin, interned_name)),
        );
    }
}

fn get_builtins() -> Vec<(&'static str, ProcedureFn)> {
    vec![
        ("+", add),
        ("*", multiply),
        ("<", less_than),
        ("define", define),
        ("lambda", lambda),
    ]
}

fn number_args(ctx: &mut ProcedureContext) -> Result<Vec<f64>, RuntimeError> {
    let mut numbers = Vec::with_capacity(ctx.operands.len());
    for expr in ctx.operands.iter() {
        numbers.push(ctx.interpreter.expect_number(expr)?);
    }
    Ok(numbers)
}

fn less_than(mut ctx: ProcedureContext) -> ProcedureResult {
    let mut latest: f64 = -INFINITY;
    for number in number_args(&mut ctx)? {
        if number <= latest {
            return Ok(Value::Boolean(false).into());
        }
        latest = number;
    }
    Ok(Value::Boolean(true).into())
}

fn add(mut ctx: ProcedureContext) -> ProcedureResult {
    let mut result = 0.0;
    for number in number_args(&mut ctx)? {
        result += number
    }
    Ok(Value::Number(result).into())
}

fn multiply(mut ctx: ProcedureContext) -> ProcedureResult {
    let mut result = 1.0;
    for number in number_args(&mut ctx)? {
        result *= number
    }
    Ok(Value::Number(result).into())
}

fn define(ctx: ProcedureContext) -> ProcedureResult {
    match ctx.operands.get(0) {
        Some(SourceMapped(ExpressionValue::Symbol(name), ..)) => {
            let mut value = ctx.interpreter.eval_expressions(&ctx.operands[1..])?;
            if let Value::Procedure(Procedure::Compound(compound)) = &mut value {
                if compound.name.is_none() {
                    compound.name = Some(name.clone());
                }
            }
            ctx.interpreter.environment.set(name.clone(), value);
            Ok(Value::Undefined.into())
        }
        Some(SourceMapped(ExpressionValue::Combination(expressions), range)) => {
            let signature = SourceMapped(expressions.clone(), *range);
            let Some(first) = signature.0.get(0) else {
                return Err(RuntimeErrorType::MalformedSpecialForm.source_mapped(signature.1));
            };
            let name = first.expect_identifier()?;
            let mut proc = CompoundProcedure::create(
                signature,
                1,
                SourceMapped(ctx.combination.0.clone(), ctx.combination.1),
                ctx.interpreter.environment.capture_lexical_scope(),
            )?;
            proc.name = Some(name.clone());
            ctx.interpreter
                .environment
                .set(name, Value::Procedure(Procedure::Compound(proc)));
            Ok(Value::Undefined.into())
        }
        _ => Err(RuntimeErrorType::MalformedSpecialForm.source_mapped(ctx.combination.1)),
    }
}

fn lambda(ctx: ProcedureContext) -> ProcedureResult {
    match ctx.operands.get(0) {
        Some(SourceMapped(ExpressionValue::Combination(expressions), range)) => {
            let signature = SourceMapped(expressions.clone(), *range);
            let proc = CompoundProcedure::create(
                signature,
                0,
                SourceMapped(ctx.combination.0.clone(), ctx.combination.1),
                ctx.interpreter.environment.capture_lexical_scope(),
            )?;
            Ok(Value::Procedure(Procedure::Compound(proc)).into())
        }
        _ => Err(RuntimeErrorType::MalformedSpecialForm.source_mapped(ctx.combination.1)),
    }
}
