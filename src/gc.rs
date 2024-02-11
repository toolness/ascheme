use std::{cell::RefCell, collections::HashSet, ops::Deref, rc::Rc};

#[derive(Default)]
pub struct Visitor {
    pub debug: bool,
    visited: RefCell<HashSet<(usize, usize)>>,
}

impl Visitor {
    /// This is pretty frustrating--all I want is a unique identifier for "the thing being traversed"
    /// so I can make sure I don't loop infinitely while traversing the object graph,
    /// but this appears to be impossible because some of the things we're traversing are
    /// actually at the same memory location as different things we're traversing (e.g.,
    /// the first item of a struct is actually at the same memory location as
    /// the struct itself).
    ///
    /// We might be able to disambiguate by the vtable of the dyn pointer, but Rust doesn't
    /// actually seem to give us access to that, and I can't store the raw pointer either,
    /// because then Rust complains about lifetime issues, so ... I guess I am just going to
    /// additionally pass in a string that represents the "type" of thing being traversed and
    /// use that as part of the identifier.
    ///
    /// This feels extremely stupid but I don't know what else to do.
    ///
    /// TODO: Maybe we only actually have to care about double-visiting
    /// Tracked objects? If that's the case, we could have a separate visit
    /// method that just takes a tracked object, and we could use its id for
    /// comparison.
    pub fn traverse(&self, traverser: &dyn Traverser, type_id: &'static str) {
        let traverser_ptr = (traverser as *const dyn Traverser) as *const () as usize;
        let type_id_ptr = (type_id as *const str) as *const () as usize;
        let id = (traverser_ptr, type_id_ptr);
        if self.visited.borrow().contains(&id) {
            if self.debug {
                println!("Already visited {type_id} @ {:#x}", traverser_ptr);
            }
            return;
        }
        if self.debug {
            println!("Visiting {type_id} @ {:#x}", traverser_ptr);
        }
        self.visited.borrow_mut().insert(id);
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
        visitor.traverse(self.borrow().deref(), "Rc");
    }
}
