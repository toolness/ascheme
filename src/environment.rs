use std::collections::HashMap;

use crate::{interpreter::Value, string_interner::InternedString};

#[derive(Default)]
pub struct Environment {
    symbols: HashMap<InternedString, Value>,
}

impl Environment {
    pub fn get(&self, identifier: &InternedString) -> Option<&Value> {
        self.symbols.get(identifier)
    }

    pub fn set(&mut self, identifier: InternedString, value: Value) {
        self.symbols.insert(identifier, value);
    }
}
