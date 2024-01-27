use std::collections::HashMap;

use crate::{
    interpreter::{Interpreter, ProcedureFn, RuntimeError, Value},
    parser::Expression,
    string_interner::{InternedString, StringInterner},
};

pub fn make_builtins(interner: &mut StringInterner) -> HashMap<InternedString, ProcedureFn> {
    let mut builtins: HashMap<InternedString, ProcedureFn> = HashMap::new();
    builtins.insert(interner.intern("+"), add);
    builtins.insert(interner.intern("*"), multiply);
    builtins
}

fn add(interpreter: &Interpreter, operands: &[Expression]) -> Result<Value, RuntimeError> {
    let mut result = 0.0;
    for expr in operands.iter() {
        let number = interpreter.expect_number(expr)?;
        result += number
    }
    Ok(Value::Number(result))
}

fn multiply(interpreter: &Interpreter, operands: &[Expression]) -> Result<Value, RuntimeError> {
    let mut result = 1.0;
    for expr in operands.iter() {
        let number = interpreter.expect_number(expr)?;
        result *= number
    }
    Ok(Value::Number(result))
}
