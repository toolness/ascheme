use std::{cell::RefCell, collections::HashSet, ops::Deref, rc::Rc};

#[derive(Default)]
pub struct Visitor {
    pub debug: bool,
    visited: RefCell<HashSet<usize>>,
}

impl Visitor {
    /// This will only traverse the given traverser if it hasn't already been
    /// traversed. It uses the traverser's pointer as its unique identifier.
    pub fn visit(&self, traverser: &dyn Traverser, name: &str) {
        let id = (traverser as *const dyn Traverser) as *const () as usize;
        if self.visited.borrow().contains(&id) {
            if self.debug {
                println!("Already visited {name} @ {id:#x}");
            }
            return;
        }
        if self.debug {
            println!("Visiting {name} @ {id:#x}");
        }
        self.visited.borrow_mut().insert(id);

        // TODO: Push/pop debug output indentation level around this call.
        traverser.traverse(self);
    }

    /// This will *always* traverse the given traverser--it doesn't actually check
    /// to see if the traverser is already traversed. (Ideally we *would* do this,
    /// but obtaining a unique identifier for the traverser is non-trivial, as
    /// e.g. the first child of a struct may have the exact same memory address as
    /// its parent.)
    pub fn traverse(&self, traverser: &dyn Traverser, _name: &'static str) {
        traverser.traverse(self);
    }
}

pub trait Traverser {
    fn traverse(&self, visitor: &Visitor);
}

impl<T: Traverser> Traverser for Vec<T> {
    fn traverse(&self, visitor: &Visitor) {
        for item in self {
            visitor.traverse(item, "Vec item");
        }
    }
}

impl<T: Traverser> Traverser for Rc<T> {
    fn traverse(&self, visitor: &Visitor) {
        visitor.traverse(self.as_ref(), "Rc");
    }
}

impl<T: Traverser> Traverser for RefCell<T> {
    fn traverse(&self, visitor: &Visitor) {
        visitor.traverse(self.borrow().deref(), "RefCell");
    }
}
