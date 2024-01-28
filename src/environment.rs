use std::collections::HashMap;

use crate::{interpreter::Value, string_interner::InternedString};

#[derive(Default)]
pub struct Environment {
    symbol_stack: Vec<HashMap<InternedString, Value>>,
}

impl Environment {
    pub fn push(&mut self) {
        self.symbol_stack.push(HashMap::new());
    }

    pub fn pop(&mut self) {
        self.symbol_stack.pop();
    }

    pub fn get(&self, identifier: &InternedString) -> Option<&Value> {
        for symbol_map in self.symbol_stack.iter().rev() {
            if let Some(value) = symbol_map.get(identifier) {
                return Some(value);
            }
        }
        None
    }

    pub fn set(&mut self, identifier: InternedString, value: Value) {
        if self.symbol_stack.is_empty() {
            self.push();
        }
        let symbols = self.symbol_stack.last_mut().unwrap();
        symbols.insert(identifier, value);
    }
}
