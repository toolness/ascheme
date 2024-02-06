use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    interpreter::SourceValue,
    source_mapped::{SourceMappable, SourceMapped, SourceRange},
    string_interner::InternedString,
};

#[derive(Default, Clone, Debug)]
struct Scope {
    parent: Option<Rc<SourceMapped<Scope>>>,
    bindings: Rc<RefCell<HashMap<InternedString, SourceValue>>>,
}

impl Scope {
    fn get(&self, identifier: &InternedString) -> Option<SourceValue> {
        if let Some(value) = self.bindings.borrow().get(identifier) {
            Some(value.clone())
        } else {
            self.parent
                .as_ref()
                .map(|parent| parent.0.get(identifier))
                .flatten()
        }
    }
}

#[derive(Debug, Clone)]
pub struct CapturedLexicalScope(Option<Rc<SourceMapped<Scope>>>);

#[derive(Default)]
pub struct Environment {
    globals: Scope,
    lexical_scopes: Vec<Rc<SourceMapped<Scope>>>,
}

impl Environment {
    pub fn clear_lexical_scopes(&mut self) {
        self.lexical_scopes.clear();
    }

    pub fn capture_lexical_scope(&self) -> CapturedLexicalScope {
        CapturedLexicalScope(self.lexical_scopes.last().cloned())
    }

    pub fn push(&mut self, scope: CapturedLexicalScope, source_range: SourceRange) {
        let mut new_scope = Scope::default();
        new_scope.parent = scope.0;
        self.lexical_scopes
            .push(new_scope.source_mapped(source_range).into());
    }

    pub fn pop(&mut self) {
        self.lexical_scopes.pop();
    }

    pub fn get(&self, identifier: &InternedString) -> Option<SourceValue> {
        if let Some(scope) = self.lexical_scopes.last() {
            if let Some(value) = scope.0.get(identifier) {
                return Some(value);
            }
        }
        self.globals.get(identifier)
    }

    pub fn set(&mut self, identifier: InternedString, value: SourceValue) {
        if let Some(scope) = self.lexical_scopes.last_mut() {
            scope.0.bindings.borrow_mut().insert(identifier, value);
        } else {
            self.globals.bindings.borrow_mut().insert(identifier, value);
        }
    }
}
