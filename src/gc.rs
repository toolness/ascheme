use std::{cell::RefCell, collections::HashSet, ops::Deref, rc::Rc};

/// A Visitor that allows the interpreter's data structures to be traversed, without
/// infinitely looping when it encounters cycles. When used on GC roots, it can be
/// used to mark all reachable objects as the first phase of a mark-and-sweep process.
#[derive(Default)]
pub struct Visitor {
    pub debug: bool,
    indent_level: RefCell<usize>,
    visited: RefCell<HashSet<usize>>,
}

impl Visitor {
    pub fn log(&self, value: &str) {
        println!("{}{}", "  ".repeat(*self.indent_level.borrow()), value);
    }

    pub fn indent(&self) {
        let indent = *self.indent_level.borrow();
        *self.indent_level.borrow_mut() = indent + 1;
    }

    pub fn dedent(&self) {
        let indent = *self.indent_level.borrow();
        *self.indent_level.borrow_mut() = indent - 1;
    }

    /// This will only traverse the given traverser if it hasn't already been
    /// traversed. It uses the traverser's pointer as its unique identifier.
    pub fn visit(&self, traverser: &dyn Traverser, name: &str) {
        let id = (traverser as *const dyn Traverser) as *const () as usize;
        if self.visited.borrow().contains(&id) {
            if self.debug {
                self.log(&format!("Already visited {name} @ {id:#x}"));
            }
            return;
        }
        if self.debug {
            self.log(&format!("Visiting {name} @ {id:#x}"));
        }
        self.visited.borrow_mut().insert(id);

        if self.debug {
            self.indent();
        }
        traverser.traverse(self);
        if self.debug {
            self.dedent();
        }
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

/// Trait implemented by objects in the interpreter, allowing `Visitor` to visit
/// all the objects in it.
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

impl<T: Traverser> Traverser for Option<T> {
    fn traverse(&self, visitor: &Visitor) {
        if let Some(traverser) = self {
            visitor.traverse(traverser, "Option");
        }
    }
}
