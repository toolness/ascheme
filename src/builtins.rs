use crate::{
    interpreter::{ProcedureContext, ProcedureFn, RuntimeError, RuntimeErrorType, Value},
    parser::ExpressionValue,
    source_mapped::{SourceMappable, SourceMapped},
    string_interner::InternedString,
};

pub fn get_builtins() -> Vec<(&'static str, ProcedureFn)> {
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
            ctx.interpreter.environment.set(*name, value);
            Ok(Value::Undefined)
        }
        Some(SourceMapped(ExpressionValue::Combination(expressions), range)) => {
            let Some(first) = expressions.get(0) else {
                return Err(RuntimeErrorType::MalformedSpecialForm.source_mapped(*range));
            };
            let name = first.expect_identifier()?;
            let mut arg_bindings: Vec<InternedString> = vec![];
            for arg_name in &expressions[1..] {
                arg_bindings.push(arg_name.expect_identifier()?);
            }

            // TODO: Do something with all this stuff!
            let _unused = name;

            Ok(Value::Undefined)
        }
        _ => Err(RuntimeErrorType::MalformedSpecialForm.source_mapped(ctx.combination.1)),
    }
}
