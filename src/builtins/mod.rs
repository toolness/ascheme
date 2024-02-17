use std::backtrace::Backtrace;

use crate::{
    compound_procedure::CompoundProcedure,
    environment::Environment,
    interpreter::{
        Interpreter, Procedure, ProcedureContext, ProcedureFn, ProcedureResult, RuntimeError,
        RuntimeErrorType,
    },
    pair::Pair,
    source_mapped::{SourceMappable, SourceMapped},
    string_interner::StringInterner,
    value::{SourceValue, Value},
};

mod math;
mod ord;
mod util;

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
        ("+", math::add),
        ("-", math::subtract),
        ("*", math::multiply),
        ("/", math::divide),
        ("<", ord::less_than),
        ("<=", ord::less_than_or_equal_to),
        (">", ord::greater_than),
        (">=", ord::greater_than_or_equal_to),
        ("=", ord::numeric_eq),
        ("define", define),
        ("lambda", lambda),
        ("quote", quote),
        ("eq?", eq),
        ("if", _if),
        ("and", and),
        ("or", or),
        ("not", not),
        ("cond", cond),
        ("set!", set),
        ("set-car!", set_car),
        ("set-cdr!", set_cdr),
        ("rust-backtrace", rust_backtrace),
        ("stats", stats),
        ("gc", gc),
        ("test-eq", test_eq),
        ("print-and-eval", print_and_eval),
    ]
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

fn and(ctx: ProcedureContext) -> ProcedureResult {
    let mut latest = true.into();
    for (i, operand) in ctx.operands.iter().enumerate() {
        if i == ctx.operands.len() - 1 {
            return ctx.interpreter.eval_expression_in_tail_context(operand);
        }
        latest = ctx.interpreter.eval_expression(operand)?.0;
        if !latest.as_bool() {
            break;
        }
    }
    Ok(latest.into())
}

fn or(ctx: ProcedureContext) -> ProcedureResult {
    let mut latest = false.into();
    for (i, operand) in ctx.operands.iter().enumerate() {
        if i == ctx.operands.len() - 1 {
            return ctx.interpreter.eval_expression_in_tail_context(operand);
        }
        latest = ctx.interpreter.eval_expression(operand)?.0;
        if latest.as_bool() {
            break;
        }
    }
    Ok(latest.into())
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

fn print_and_eval(ctx: ProcedureContext) -> ProcedureResult {
    if ctx.operands.len() != 1 {
        return Err(RuntimeErrorType::WrongNumberOfArguments.source_mapped(ctx.combination.1));
    }
    let operand_repr = ctx.operands[0].to_string();
    let value = ctx.interpreter.eval_expression(&ctx.operands[0])?;
    println!("{} = {}", operand_repr, value);
    Ok(value.into())
}

fn test_eq(mut ctx: ProcedureContext) -> ProcedureResult {
    if ctx.operands.len() != 2 {
        return Err(RuntimeErrorType::WrongNumberOfArguments.source_mapped(ctx.combination.1));
    }
    let operand_0_repr = ctx.operands[0].to_string();
    let operand_1_repr = ctx.operands[1].to_string();

    if is_eq(&mut ctx.interpreter, &ctx.operands[0], &ctx.operands[1])? {
        println!("OK {} = {}", operand_0_repr, operand_1_repr);
    } else {
        println!("ERR {} != {}", operand_0_repr, operand_1_repr);
    }

    Ok(Value::Undefined.into())
}

fn is_eq(
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

fn eq(mut ctx: ProcedureContext) -> ProcedureResult {
    if ctx.operands.len() < 2 {
        return Err(RuntimeErrorType::WrongNumberOfArguments.source_mapped(ctx.combination.1));
    }

    Ok(is_eq(&mut ctx.interpreter, &ctx.operands[0], &ctx.operands[1])?.into())
}
