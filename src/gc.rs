use std::{cell::RefCell, collections::HashSet, rc::Rc};

#[derive(Default)]
pub struct Visitor {
    visited: RefCell<HashSet<(usize, usize)>>,
}

impl Visitor {
    pub fn traverse(&self, traverser: &dyn Traverser, type_id: &'static str) {
        let traverser_ptr = (traverser as *const dyn Traverser) as *const () as usize;
        let type_id_ptr = (type_id as *const str) as *const () as usize;
        let id = (traverser_ptr, type_id_ptr);
        if self.visited.borrow().contains(&id) {
            println!("Already visited {type_id} @ {:#x}", traverser_ptr);
            return;
        }
        println!("Visiting {type_id} @ {:#x}", traverser_ptr);
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
        visitor.traverse(self, "Rc");
    }
}
