use std::collections::HashMap;

use crate::{interpreter::Value, string_interner::InternedString};

type Bindings = HashMap<InternedString, Value>;

#[derive(Default)]
pub struct Environment {
    globals: Bindings,
    lexical_scope_stack: Vec<Bindings>,
}

impl Environment {
    pub fn push(&mut self) {
        self.lexical_scope_stack.push(HashMap::new());
    }

    pub fn pop(&mut self) {
        self.lexical_scope_stack.pop();
    }

    pub fn get(&self, identifier: &InternedString) -> Option<Value> {
        for bindings in self.lexical_scope_stack.iter().rev() {
            if let Some(value) = bindings.get(identifier) {
                return Some(value.clone());
            }
        }
        self.globals.get(identifier).map(|value| value.clone())
    }

    pub fn set(&mut self, identifier: InternedString, value: Value) {
        if let Some(bindings) = self.lexical_scope_stack.last_mut() {
            bindings.insert(identifier, value);
        } else {
            self.globals.insert(identifier, value);
        }
    }
}
