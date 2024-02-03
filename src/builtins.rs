use crate::{
    compound_procedure::CompoundProcedure,
    environment::Environment,
    interpreter::{
        Procedure, ProcedureContext, ProcedureFn, RuntimeError, RuntimeErrorType, Value,
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
    vec![("+", add), ("*", multiply), ("define", define)]
}

fn add(ctx: ProcedureContext) -> Result<Value, RuntimeError> {
    let mut result = 0.0;
    for expr in ctx.operands.iter() {
        let number = ctx.interpreter.expect_number(expr)?;
        result += number
    }
    Ok(Value::Number(result))
}

fn multiply(ctx: ProcedureContext) -> Result<Value, RuntimeError> {
    let mut result = 1.0;
    for expr in ctx.operands.iter() {
        let number = ctx.interpreter.expect_number(expr)?;
        result *= number
    }
    Ok(Value::Number(result))
}

fn define(ctx: ProcedureContext) -> Result<Value, RuntimeError> {
    match ctx.operands.get(0) {
        Some(SourceMapped(ExpressionValue::Symbol(name), ..)) => {
            let value = ctx.interpreter.eval_expressions(&ctx.operands[1..])?;
            ctx.interpreter.environment.set(name.clone(), value);
            Ok(Value::Undefined)
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
            Ok(Value::Undefined)
        }
        _ => Err(RuntimeErrorType::MalformedSpecialForm.source_mapped(ctx.combination.1)),
    }
}
