use std::{cell::RefCell, fmt::Display, rc::Rc};

#[derive(Debug, Clone)]
pub struct MutableString(Rc<RefCell<String>>);

impl MutableString {
    pub fn new(value: String) -> Self {
        MutableString(Rc::new(RefCell::new(value)))
    }

    pub fn points_at_same_memory_as(&self, other: &MutableString) -> bool {
        &*self.0 as *const RefCell<String> == &*other.0 as *const RefCell<String>
    }

    pub fn repr(&self) -> String {
        format!("{:?}", self.0.borrow().as_str())
    }
}

impl Display for MutableString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.borrow().as_str())
    }
}
