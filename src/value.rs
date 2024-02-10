use std::{fmt::Display, rc::Rc};

use crate::{
    interpreter::{Procedure, RuntimeError, RuntimeErrorType},
    pair::Pair,
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

    pub fn expect_pair(&self) -> Result<Pair, RuntimeError> {
        println!("UM expect_pair {:?}", self);
        if let Value::Pair(pair) = &self.0 {
            Ok(pair.clone())
        } else {
            Err(RuntimeErrorType::ExpectedPair.source_mapped(self.1))
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
    EmptyList,
    Number(f64),
    Symbol(InternedString),
    Boolean(bool),
    Procedure(Procedure),
    Pair(Pair),
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

impl SourceMapped<Value> {
    pub fn try_into_list(&self) -> Option<SourceMapped<Rc<Vec<SourceValue>>>> {
        match self {
            SourceMapped(Value::Pair(pair), range) => {
                let Some(expressions) = pair.try_as_rc_list() else {
                    return None;
                };
                Some(SourceMapped(expressions, *range))
            }
            SourceMapped(Value::EmptyList, range) => Some(SourceMapped(vec![].into(), *range)),
            _ => None,
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Undefined => write!(f, ""),
            Value::EmptyList => write!(f, "()"),
            Value::Number(value) => write!(f, "{}", value),
            Value::Symbol(name) => write!(f, "{}", name),
            Value::Pair(pair) => {
                match pair.try_get_vec_pair() {
                    Some(vec_pair) => write!(f, "{}", vec_pair),
                    None => {
                        // TODO: Implement display for cyclic lists.
                        write!(f, "<CYCLIC LIST>")
                    }
                }
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
