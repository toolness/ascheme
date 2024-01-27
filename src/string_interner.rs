use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InternedString(u32);

#[derive(Default)]
pub struct StringInterner {
    strings_to_ids: HashMap<String, InternedString>,
    ids_to_strings: Vec<String>,
}

impl StringInterner {
    pub fn intern<T: AsRef<str>>(&mut self, value: T) -> InternedString {
        if let Some(&id) = self.strings_to_ids.get(value.as_ref()) {
            id
        } else {
            let id = InternedString(self.ids_to_strings.len() as u32);
            self.strings_to_ids.insert(value.as_ref().to_string(), id);
            self.ids_to_strings.push(value.as_ref().to_string());
            id
        }
    }

    pub fn get(&self, id: InternedString) -> &str {
        self.ids_to_strings.get(id.0 as usize).unwrap().as_str()
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
        assert_eq!(interner.get(boop1), "boop");
        assert_eq!(interner.get(bap), "bap");
    }
}
