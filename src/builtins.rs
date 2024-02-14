use std::{backtrace::Backtrace, f64::INFINITY};

use crate::{
    compound_procedure::CompoundProcedure,
    environment::Environment,
    interpreter::{
        Procedure, ProcedureContext, ProcedureFn, ProcedureResult, RuntimeError, RuntimeErrorType,
    },
    pair::Pair,
    source_mapped::{SourceMappable, SourceMapped},
    string_interner::StringInterner,
    value::{SourceValue, Value},
};

pub fn populate_environment(environment: &mut Environment, interner: &mut StringInterner) {
    for (name, builtin) in get_builtins() {
        let interned_name = interner.intern(name);
        environment.define(
            interned_name.clone(),
            Value::Procedure(Procedure::Builtin(builtin, interned_name)).into(),
        );
    }
    // TODO: Technically 'else' is just part of how the 'cond' special form is evaluated,
    // but just aliasing it to 'true' is easier for now.
    environment.define(interner.intern("else"), Value::Boolean(true).into());
}

fn get_builtins() -> Vec<(&'static str, ProcedureFn)> {
    vec![
        ("+", add),
        ("-", subtract),
        ("*", multiply),
        ("/", divide),
        ("<", less_than),
        ("<=", less_than_or_equal_to),
        (">", greater_than),
        (">=", greater_than_or_equal_to),
        ("=", numeric_eq),
        ("define", define),
        ("lambda", lambda),
        ("quote", quote),
        ("if", _if),
        ("not", not),
        ("cond", cond),
        ("set!", set),
        ("set-car!", set_car),
        ("set-cdr!", set_cdr),
        ("rust-backtrace", rust_backtrace),
        ("stats", stats),
        ("gc", gc),
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
            return Ok(false.into());
        }
        latest = number;
    }
    Ok(true.into())
}

fn less_than_or_equal_to(mut ctx: ProcedureContext) -> ProcedureResult {
    let mut latest: f64 = -INFINITY;
    for number in number_args(&mut ctx)? {
        if number < latest {
            return Ok(false.into());
        }
        latest = number;
    }
    Ok(true.into())
}

fn greater_than(mut ctx: ProcedureContext) -> ProcedureResult {
    let mut latest: f64 = INFINITY;
    for number in number_args(&mut ctx)? {
        if number >= latest {
            return Ok(false.into());
        }
        latest = number;
    }
    Ok(true.into())
}

fn greater_than_or_equal_to(mut ctx: ProcedureContext) -> ProcedureResult {
    let mut latest: f64 = INFINITY;
    for number in number_args(&mut ctx)? {
        if number > latest {
            return Ok(false.into());
        }
        latest = number;
    }
    Ok(true.into())
}

fn add(mut ctx: ProcedureContext) -> ProcedureResult {
    let mut result = 0.0;
    for number in number_args(&mut ctx)? {
        result += number
    }
    Ok(result.into())
}

fn subtract(mut ctx: ProcedureContext) -> ProcedureResult {
    let numbers = number_args(&mut ctx)?;
    if numbers.len() == 0 {
        return Err(RuntimeErrorType::WrongNumberOfArguments.source_mapped(ctx.combination.1));
    }
    let mut result = numbers[0];
    if numbers.len() == 1 {
        return Ok((-result).into());
    }
    for number in &numbers[1..] {
        result -= number
    }
    Ok(result.into())
}

fn multiply(mut ctx: ProcedureContext) -> ProcedureResult {
    let mut result = 1.0;
    for number in number_args(&mut ctx)? {
        result *= number
    }
    Ok(result.into())
}

fn divide(mut ctx: ProcedureContext) -> ProcedureResult {
    let numbers = number_args(&mut ctx)?;
    if numbers.len() == 0 {
        return Err(RuntimeErrorType::WrongNumberOfArguments.source_mapped(ctx.combination.1));
    }
    let mut result = 1.0;
    for number in numbers {
        if number == 0.0 {
            // Ideally we'd point at the specific argument that's zero, but this is good enough for now.
            return Err(RuntimeErrorType::DivisionByZero.source_mapped(ctx.combination.1));
        }
        result = result / number;
    }
    Ok(result.into())
}

fn numeric_eq(mut ctx: ProcedureContext) -> ProcedureResult {
    let numbers = number_args(&mut ctx)?;
    if numbers.len() < 2 {
        Ok(true.into())
    } else {
        let number = numbers[0];
        for other_number in &numbers[1..] {
            if *other_number != number {
                return Ok(false.into());
            }
        }
        Ok(true.into())
    }
}

fn _if(ctx: ProcedureContext) -> ProcedureResult {
    if ctx.operands.len() < 2 || ctx.operands.len() > 3 {
        return Err(RuntimeErrorType::MalformedSpecialForm.source_mapped(ctx.combination.1));
    }
    let test = ctx.interpreter.eval_expression(&ctx.operands[0])?.0;
    if test.as_bool() {
        let consequent_expr = &ctx.operands[1];
        ctx.interpreter
            .eval_expression_in_tail_context(consequent_expr)
    } else {
        if let Some(alternate_expr) = ctx.operands.get(2) {
            ctx.interpreter
                .eval_expression_in_tail_context(alternate_expr)
        } else {
            Ok(Value::Undefined.into())
        }
    }
}

fn not(ctx: ProcedureContext) -> ProcedureResult {
    if ctx.operands.len() != 1 {
        return Err(RuntimeErrorType::WrongNumberOfArguments.source_mapped(ctx.combination.1));
    }
    let result = ctx.interpreter.eval_expression(&ctx.operands[0])?.0;
    Ok((!result.as_bool()).into())
}

fn cond(ctx: ProcedureContext) -> ProcedureResult {
    if ctx.operands.len() == 0 {
        return Err(RuntimeErrorType::MalformedSpecialForm.source_mapped(ctx.combination.1));
    }

    for clause in ctx.operands.iter() {
        let SourceMapped(Value::Pair(pair), range) = clause else {
            return Err(RuntimeErrorType::MalformedSpecialForm.source_mapped(clause.1));
        };
        let Some(clause) = pair.try_as_rc_list() else {
            return Err(RuntimeErrorType::MalformedSpecialForm.source_mapped(*range));
        };
        let test = ctx.interpreter.eval_expression(&clause[0])?.0;
        if test.as_bool() {
            if clause.len() == 1 {
                return Ok(test.into());
            }
            return ctx
                .interpreter
                .eval_expressions_in_tail_context(&clause[1..]);
        }
    }

    Ok(Value::Undefined.into())
}

fn define(ctx: ProcedureContext) -> ProcedureResult {
    match ctx.operands.get(0) {
        Some(SourceMapped(Value::Symbol(name), ..)) => {
            let mut value = ctx.interpreter.eval_expressions(&ctx.operands[1..])?;
            if let Value::Procedure(Procedure::Compound(compound)) = &mut value.0 {
                if compound.name.is_none() {
                    compound.name = Some(name.clone());
                }
            }
            ctx.interpreter.environment.define(name.clone(), value);
            Ok(Value::Undefined.into())
        }
        Some(SourceMapped(Value::Pair(pair), range)) => {
            let Some(expressions) = pair.try_as_rc_list() else {
                return Err(RuntimeErrorType::MalformedSpecialForm.source_mapped(*range));
            };
            let signature = SourceMapped(expressions, *range);
            // We can just unwrap this b/c it's from a pair.
            let first = signature.0.get(0).unwrap();
            let name = first.expect_identifier()?;
            let mut proc = CompoundProcedure::create(
                ctx.interpreter.new_id(),
                signature,
                1,
                SourceMapped(ctx.combination.0.clone(), ctx.combination.1),
                ctx.interpreter.environment.capture_lexical_scope(),
            )?;
            proc.name = Some(name.clone());
            ctx.interpreter.environment.define(
                name,
                Value::Procedure(Procedure::Compound(proc)).source_mapped(*range),
            );
            Ok(Value::Undefined.into())
        }
        _ => Err(RuntimeErrorType::MalformedSpecialForm.source_mapped(ctx.combination.1)),
    }
}

fn lambda(ctx: ProcedureContext) -> ProcedureResult {
    let Some(SourceMapped(expressions, range)) = ctx
        .operands
        .get(0)
        .map(|value| value.try_into_list())
        .flatten()
    else {
        return Err(RuntimeErrorType::MalformedSpecialForm.source_mapped(ctx.combination.1));
    };
    let signature = SourceMapped(expressions.clone(), range);
    let proc = CompoundProcedure::create(
        ctx.interpreter.new_id(),
        signature,
        0,
        SourceMapped(ctx.combination.0.clone(), ctx.combination.1),
        ctx.interpreter.environment.capture_lexical_scope(),
    )?;
    Ok(Value::Procedure(Procedure::Compound(proc)).into())
}

fn quote(ctx: ProcedureContext) -> ProcedureResult {
    if ctx.operands.len() == 1 {
        Ok(ctx.operands[0].clone().into())
    } else {
        Err(RuntimeErrorType::MalformedSpecialForm.source_mapped(ctx.combination.1))
    }
}

fn rust_backtrace(ctx: ProcedureContext) -> ProcedureResult {
    println!(
        "Rust backtrace at {}",
        ctx.interpreter
            .source_mapper
            .trace(&ctx.combination.1)
            .join("\n")
    );
    println!("{}", Backtrace::force_capture());
    ctx.interpreter
        .eval_expressions_in_tail_context(ctx.operands)
}

fn eval_pair_and_value(ctx: &mut ProcedureContext) -> Result<(Pair, SourceValue), RuntimeError> {
    if ctx.operands.len() != 2 {
        return Err(RuntimeErrorType::WrongNumberOfArguments.source_mapped(ctx.combination.1));
    }
    let pair = ctx
        .interpreter
        .eval_expression(&ctx.operands[0])?
        .expect_pair()?;
    let value = ctx.interpreter.eval_expression(&ctx.operands[1])?;
    Ok((pair, value))
}

fn set(ctx: ProcedureContext) -> ProcedureResult {
    if ctx.operands.len() != 2 {
        return Err(RuntimeErrorType::WrongNumberOfArguments.source_mapped(ctx.combination.1));
    }
    let identifier = ctx.operands[0].expect_identifier()?;
    let value = ctx.interpreter.eval_expression(&ctx.operands[1])?;
    if let Err(err) = ctx.interpreter.environment.change(&identifier, value) {
        Err(err.source_mapped(ctx.operands[0].1))
    } else {
        Ok(Value::Undefined.into())
    }
}

fn set_car(mut ctx: ProcedureContext) -> ProcedureResult {
    let (mut pair, value) = eval_pair_and_value(&mut ctx)?;
    pair.set_car(value);
    Ok(Value::Undefined.into())
}

fn set_cdr(mut ctx: ProcedureContext) -> ProcedureResult {
    let (mut pair, value) = eval_pair_and_value(&mut ctx)?;
    pair.set_cdr(value);
    Ok(Value::Undefined.into())
}

fn stats(ctx: ProcedureContext) -> ProcedureResult {
    ctx.interpreter.print_stats();
    Ok(Value::Undefined.into())
}

fn gc(ctx: ProcedureContext) -> ProcedureResult {
    let objs_found_in_cycles = ctx.interpreter.gc(true);
    Ok((objs_found_in_cycles as f64).into())
}
