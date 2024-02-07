use std::{fmt::Display, rc::Rc};

use crate::{
    interpreter::{Procedure, RuntimeError, RuntimeErrorType},
    source_mapped::{SourceMappable, SourceMapped},
    string_interner::InternedString,
};

impl SourceMapped<Value> {
    pub fn expect_identifier(&self) -> Result<InternedString, RuntimeError> {
        if let Value::Symbol(symbol) = &self.0 {
            Ok(symbol.clone())
        } else {
            Err(RuntimeErrorType::ExpectedIdentifier.source_mapped(self.1))
        }
    }
}

pub type SourceValue = SourceMapped<Value>;

impl<T: Into<Value>> From<T> for SourceValue {
    fn from(value: T) -> Self {
        value.into().empty_source_map()
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Undefined,
    Number(f64),
    Symbol(InternedString),
    Boolean(bool),
    Procedure(Procedure),
    List(Rc<Vec<SourceValue>>),
}

impl Value {
    /// From R5RS 6.3.1:
    ///
    /// > Of all the standard Scheme values, only `#f` counts as false
    /// > in conditional expressions. Except for `#f`, all standard
    /// > Scheme values, including `#t`, pairs, the empty list, symbols,
    /// > numbers, strings, vectors, and procedures, count as true.
    pub fn as_bool(&self) -> bool {
        match self {
            Value::Boolean(false) => false,
            _ => true,
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Undefined => write!(f, ""),
            Value::Number(value) => write!(f, "{}", value),
            Value::Symbol(name) => write!(f, "{}", name),
            Value::List(items) => {
                write!(f, "(")?;
                let len = items.len();
                for (i, item) in items.iter().enumerate() {
                    write!(f, "{}", item)?;
                    if i < len - 1 {
                        write!(f, " ")?;
                    }
                }
                write!(f, ")")
            }
            Value::Boolean(boolean) => write!(f, "{}", if *boolean { "#t" } else { "#f" }),
            Value::Procedure(Procedure::Builtin(_, name)) => {
                write!(f, "#<builtin procedure {}>", name.as_ref())
            }
            Value::Procedure(Procedure::Compound(compound)) => write!(
                f,
                "#<procedure{} #{}>",
                match &compound.name {
                    Some(name) => format!(" {}", name.as_ref()),
                    None => format!(""),
                },
                compound.id()
            ),
        }
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Boolean(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::Number(value)
    }
}
