#[derive(Debug)]
pub struct SourceMapped<T>(pub T, pub (usize, usize));

impl<T> SourceMapped<T> {
    pub fn source<'a>(&self, source: &'a str) -> &'a str {
        let (start, end) = self.1;
        &source[start..end]
    }
}
