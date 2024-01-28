use crate::string_interner::InternedString;

#[derive(Debug)]
pub struct SourceMapped<T>(pub T, pub (usize, usize, Option<InternedString>));

impl<T> SourceMapped<T> {
    pub fn source<'a>(&self, source: &'a str) -> &'a str {
        let (start, end, ..) = self.1;
        &source[start..end]
    }

    /// Returns a range extending from the beginning of this item's
    /// range to the end of the given item's range.
    pub fn extend_range(&self, other: &SourceMapped<T>) -> (usize, usize, Option<InternedString>) {
        assert_eq!(self.1 .2, other.1 .2, "Ranges must be from the same file");
        (self.1 .0, other.1 .1, self.1 .2)
    }
}

pub trait SourceMappable {
    fn source_mapped(self, range: (usize, usize, Option<InternedString>)) -> SourceMapped<Self>
    where
        Self: Sized,
    {
        SourceMapped(self, range)
    }

    /// Use of this is to be avoided if possible. Only use it when we don't have
    /// access to the source map or it's not applicable for some reason.
    fn empty_source_map(self) -> SourceMapped<Self>
    where
        Self: Sized,
    {
        SourceMapped(self, (0, 0, None))
    }
}

impl<T: Sized> SourceMappable for T {}
