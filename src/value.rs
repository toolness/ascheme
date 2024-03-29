use std::{fmt::Display, rc::Rc};

use crate::{
    callable::Callable,
    gc::{Traverser, Visitor},
    interpreter::{RuntimeError, RuntimeErrorType},
    mutable_string::MutableString,
    pair::Pair,
    procedure::Procedure,
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

    pub fn expect_number(&self) -> Result<f64, RuntimeError> {
        if let Value::Number(number) = self.0 {
            Ok(number)
        } else {
            Err(RuntimeErrorType::ExpectedNumber.source_mapped(self.1))
        }
    }

    pub fn expect_pair(&self) -> Result<Pair, RuntimeError> {
        if let Value::Pair(pair) = &self.0 {
            Ok(pair.clone())
        } else {
            Err(RuntimeErrorType::ExpectedPair.source_mapped(self.1))
        }
    }

    pub fn expect_procedure(&self) -> Result<Procedure, RuntimeError> {
        if let Value::Callable(Callable::Procedure(procedure)) = &self.0 {
            Ok(procedure.clone())
        } else {
            Err(RuntimeErrorType::ExpectedProcedure.source_mapped(self.1))
        }
    }

    pub fn expect_list(&self) -> Result<Rc<Vec<SourceValue>>, RuntimeError> {
        match self.try_into_list() {
            Some(list) => Ok(list.0),
            None => Err(RuntimeErrorType::ExpectedList.source_mapped(self.1)),
        }
    }
}

pub type SourceValue = SourceMapped<Value>;

impl<T: Into<Value>> From<T> for SourceValue {
    fn from(value: T) -> Self {
        value.into().empty_source_map()
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Undefined,
    EmptyList,
    Number(f64),
    Symbol(InternedString),
    Boolean(bool),
    String(MutableString),
    Callable(Callable),
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

impl Traverser for Value {
    fn traverse(&self, visitor: &Visitor) {
        match self {
            Value::Pair(pair) => {
                visitor.traverse(pair);
            }
            Value::Callable(Callable::Procedure(Procedure::Compound(compound))) => {
                visitor.traverse(compound);
            }
            _ => {}
        }
    }
}

impl Display for Value {
    /// This displays a representation of the value as it would
    /// ordinarily be shown in a REPL.
    ///
    /// If in alternate mode (i.e., the `#` flag was specified), displays
    /// a representation that would be shown via the `display` function (e.g.,
    /// strings are not shown with quotes around them).
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Undefined => write!(f, "#!void"),
            Value::EmptyList => write!(f, "()"),
            Value::Number(value) => write!(f, "{}", value),
            Value::Symbol(name) => write!(f, "{}", name),
            Value::String(string) => {
                if f.alternate() {
                    string.fmt(f)
                } else {
                    write!(f, "{}", string.repr())
                }
            }
            Value::Pair(pair) => {
                match pair.try_get_vec_pair() {
                    Some(vec_pair) => vec_pair.fmt(f),
                    None => {
                        // TODO: Implement display for cyclic lists.
                        write!(f, "<CYCLIC LIST>")
                    }
                }
            }
            Value::Boolean(boolean) => write!(f, "{}", if *boolean { "#t" } else { "#f" }),
            Value::Callable(Callable::SpecialForm(special_form)) => {
                write!(f, "#<special form {}>", special_form.name.as_ref())
            }
            Value::Callable(Callable::Procedure(Procedure::Builtin(builtin))) => {
                write!(f, "#<builtin procedure {}>", builtin.name.as_ref())
            }
            Value::Callable(Callable::Procedure(Procedure::Compound(compound))) => write!(
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
