use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    gc::{Traverser, Visitor},
    object_tracker::{CycleBreaker, ObjectTracker, Tracked},
    source_mapped::{SourceMappable, SourceMapped, SourceRange},
    string_interner::InternedString,
    value::SourceValue,
};

#[derive(Default, Clone, Debug)]
struct Scope {
    parent: Option<Tracked<SourceMapped<Scope>>>,
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

impl CycleBreaker for Scope {
    fn break_cycles(&self) {
        self.bindings.borrow_mut().clear();
    }

    fn debug_name(&self) -> &'static str {
        "Scope"
    }
}

impl Traverser for Scope {
    fn traverse(&self, visitor: &Visitor) {
        if let Some(parent) = &self.parent {
            visitor.traverse(parent);
        }
        for (name, value) in self.bindings.borrow().iter() {
            if visitor.debug {
                visitor.log(&format!("Traversing scope binding: {}", name));
                visitor.indent();
            }
            visitor.traverse(value);
            if visitor.debug {
                visitor.dedent();
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct CapturedLexicalScope(Option<Tracked<SourceMapped<Scope>>>);

impl Traverser for CapturedLexicalScope {
    fn traverse(&self, visitor: &Visitor) {
        if let Some(scope) = &self.0 {
            visitor.traverse(scope);
        }
    }
}

#[derive(Default)]
pub struct Environment {
    globals: Scope,
    lexical_scopes: Vec<Tracked<SourceMapped<Scope>>>,
    tracker: ObjectTracker<SourceMapped<Scope>>,
}

impl Environment {
    pub fn print_stats(&self) {
        println!("Lexical scopes: {}", self.tracker.stats());
    }

    pub fn begin_mark(&mut self) {
        self.tracker.begin_mark();
    }

    pub fn sweep(&mut self) -> usize {
        self.tracker.sweep()
    }

    pub fn clear_lexical_scopes(&mut self) {
        self.lexical_scopes.clear();
    }

    pub fn capture_lexical_scope(&self) -> CapturedLexicalScope {
        CapturedLexicalScope(self.lexical_scopes.last().cloned())
    }

    pub fn push(&mut self, scope: CapturedLexicalScope, source_range: SourceRange) {
        let mut new_scope = Scope::default();
        new_scope.parent = scope.0;
        let tracked_scope = self.tracker.track(new_scope.source_mapped(source_range));
        self.lexical_scopes.push(tracked_scope);
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

    /// This works like the `define` Scheme builtin, which creates/sets the value at the
    /// current scope--it will *not* modify an existing binding in a parent lexical scope.
    pub fn define(&mut self, identifier: InternedString, value: SourceValue) {
        if let Some(scope) = self.lexical_scopes.last_mut() {
            scope.0.bindings.borrow_mut().insert(identifier, value);
        } else {
            self.globals.bindings.borrow_mut().insert(identifier, value);
        }
    }
}

impl Traverser for Environment {
    fn traverse(&self, visitor: &Visitor) {
        visitor.traverse(&self.globals);
        visitor.traverse(&self.lexical_scopes);
    }
}
