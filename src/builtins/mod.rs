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

mod eq;
mod library;
mod logic;
mod math;
mod non_standard;
mod ord;
mod util;

pub use library::add_library_source;

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

pub type Builtins = Vec<(&'static str, ProcedureFn)>;

fn get_builtins() -> Builtins {
    let mut builtins: Builtins = vec![
        ("define", define),
        ("lambda", lambda),
        ("quote", quote),
        ("display", display),
        ("if", _if),
        ("cond", cond),
        ("set!", set),
        ("set-car!", set_car),
        ("set-cdr!", set_cdr),
    ];
    builtins.extend(math::get_builtins());
    builtins.extend(eq::get_builtins());
    builtins.extend(ord::get_builtins());
    builtins.extend(logic::get_builtins());
    builtins.extend(non_standard::get_builtins());
    builtins
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

fn display(ctx: ProcedureContext) -> ProcedureResult {
    if ctx.operands.len() != 1 {
        return Err(RuntimeErrorType::WrongNumberOfArguments.source_mapped(ctx.combination.1));
    }
    let value = ctx.interpreter.eval_expression(&ctx.operands[0])?;
    ctx.interpreter.printer.print(format!("{:#}", value));
    Ok(Value::Undefined.into())
}
