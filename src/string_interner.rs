use core::fmt::Debug;
use core::hash::Hash;
use std::{collections::HashMap, fmt::Display, rc::Rc};

// The first u32 is really the most important information here,
// the Rc<String> is essentially denormalized data that's packaged
// with the struct for convenience. All equality/hash/etc operations
// can work only on the id.
#[derive(Clone)]
pub struct InternedString(u32, Rc<String>);

impl AsRef<str> for InternedString {
    fn as_ref(&self) -> &str {
        self.1.as_str()
    }
}

impl PartialEq for InternedString {
    fn eq(&self, other: &Self) -> bool {
        // Note that we ignore the Rc<String>, comparing id is enough!
        self.0 == other.0
    }
}

impl Eq for InternedString {}

impl Hash for InternedString {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Note that we ignore the Rc<String>, id is enough!
        self.0.hash(state);
    }
}

impl Debug for InternedString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} (#{})", self.1.as_str(), self.0)
    }
}

impl Display for InternedString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.1)
    }
}

#[derive(Default)]
pub struct StringInterner {
    // TODO: This isn't great, we're allocating 2x more strings than we need to,
    // but it makes the borrow checker happy and it's good enough for now.
    //
    // This is partially taken from:
    // https://matklad.github.io/2020/03/22/fast-simple-rust-interner.html
    strings_to_ids: HashMap<String, u32>,
    ids_to_strings: Vec<Rc<String>>,
}

impl StringInterner {
    pub fn intern<T: AsRef<str>>(&mut self, value: T) -> InternedString {
        if let Some(&id) = self.strings_to_ids.get(value.as_ref()) {
            InternedString(id, self.ids_to_strings.get(id as usize).unwrap().clone())
        } else {
            let id = self.ids_to_strings.len() as u32;
            let string = value.as_ref().to_string();
            let rc_string = Rc::new(string.clone());
            self.strings_to_ids.insert(string, id);
            self.ids_to_strings.push(rc_string.clone());
            InternedString(id, rc_string)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::string_interner::StringInterner;

    #[test]
    fn it_works() {
        let mut interner = StringInterner::default();

        let boop1 = interner.intern("boop");
        let boop2 = interner.intern("boop");
        let bap = interner.intern("bap");

        assert_eq!(boop1, boop2);
        assert_ne!(boop1, bap);
        assert_eq!(boop1.as_ref(), "boop");
        assert_eq!(bap.as_ref(), "bap");
    }
}
