use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{interpreter::Value, string_interner::InternedString};

#[derive(Default, Clone, Debug)]
struct Scope {
    parent: Option<Rc<Scope>>,
    bindings: Rc<RefCell<HashMap<InternedString, Value>>>,
}

impl Scope {
    fn get(&self, identifier: &InternedString) -> Option<Value> {
        if let Some(value) = self.bindings.borrow().get(identifier) {
            Some(value.clone())
        } else {
            self.parent
                .as_ref()
                .map(|parent| parent.get(identifier))
                .flatten()
        }
    }
}

#[derive(Debug, Clone)]
pub struct CapturedLexicalScope(Rc<Scope>);

#[derive(Default)]
pub struct Environment {
    globals: Scope,
    lexical_scopes: Vec<Rc<Scope>>,
}

impl Environment {
    pub fn capture_lexical_scope(&self) -> CapturedLexicalScope {
        CapturedLexicalScope(self.lexical_scopes.last().cloned().unwrap_or_default())
    }

    pub fn push(&mut self, scope: CapturedLexicalScope) {
        let mut new_scope = Scope::default();
        new_scope.parent = Some(scope.0);
        self.lexical_scopes.push(new_scope.into());
    }

    pub fn pop(&mut self) {
        self.lexical_scopes.pop();
    }

    pub fn get(&self, identifier: &InternedString) -> Option<Value> {
        if let Some(scope) = self.lexical_scopes.last() {
            if let Some(value) = scope.get(identifier) {
                return Some(value);
            }
        }
        self.globals.get(identifier)
    }

    pub fn set(&mut self, identifier: InternedString, value: Value) {
        if let Some(scope) = self.lexical_scopes.last_mut() {
            scope.bindings.borrow_mut().insert(identifier, value);
        } else {
            self.globals.bindings.borrow_mut().insert(identifier, value);
        }
    }
}
