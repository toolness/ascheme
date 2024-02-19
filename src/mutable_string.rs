use std::{cell::RefCell, fmt::Display, rc::Rc};

#[derive(Debug, Clone)]
pub struct MutableString(Rc<RefCell<String>>);

impl MutableString {
    pub fn new(value: String) -> Self {
        MutableString(Rc::new(RefCell::new(value)))
    }

    pub fn from_tokenized_source(repr: &str) -> Self {
        let mut chars: Vec<char> = Vec::with_capacity(repr.len());
        // The `skip(1)` skips the opening quote.
        let mut is_escaped = false;
        for char in repr.chars().skip(1) {
            if is_escaped {
                if char == 'n' {
                    chars.push('\n');
                } else {
                    chars.push(char);
                }
                is_escaped = false;
            } else {
                if char == '\\' {
                    is_escaped = true;
                } else {
                    chars.push(char);
                }
            }
        }
        chars.pop(); // Remove closing quote.
        let string: String = chars.into_iter().collect();
        Self::new(string)
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
