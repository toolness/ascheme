use std::{cmp::min, collections::HashMap};

use crate::source_mapped::SourceRange;

#[derive(Debug, Hash, Eq, PartialEq, Copy, Clone)]
pub struct SourceId(usize);

#[derive(Debug, PartialEq)]
pub struct MappedLine<'a> {
    line_number: usize,
    start: usize,
    end: usize,
    line: &'a str,
}

impl<'a> MappedLine<'a> {
    fn new(line_number: usize, start: usize, end: usize, line: &'a str) -> Self {
        MappedLine {
            line_number,
            start,
            end,
            line,
        }
    }

    fn from_source(contents: &'a str, start: usize, end: usize) -> Option<Self> {
        let mut latest_char = 0;
        for (i, line) in contents.lines().enumerate() {
            if latest_char + line.len() > start {
                let rel_start = start - latest_char;
                let rel_end = min(rel_start + (end - start), line.len());
                return Some(MappedLine::new(i, rel_start, rel_end, line));
            }
            // Add 1 for the newline character at the end.
            latest_char += line.len() + 1;
        }
        None
    }
}

pub struct Source {
    filename: String,
    contents: String,
}

#[derive(Default)]
pub struct SourceMapper {
    sources: HashMap<usize, Source>,
    latest_id: usize,
}

impl SourceMapper {
    pub fn add(&mut self, filename: String, contents: String) -> SourceId {
        let id = self.latest_id;
        self.sources.insert(id, Source { filename, contents });
        self.latest_id += 1;
        SourceId(id)
    }

    pub fn get_contents(&self, id: SourceId) -> &str {
        &self.sources.get(&id.0).unwrap().contents
    }

    /// Given a source range, return the line number it's on, the start position of
    /// the range within the line, and the end position of the range within the line.
    /// If the range extends past the line, the end position will be the end of the line.
    pub fn get_first_line(&self, source_range: &SourceRange) -> Option<MappedLine> {
        let &(start, end, Some(source_id)) = source_range else {
            return None;
        };
        let contents = self.get_contents(source_id);
        MappedLine::from_source(contents, start, end)
    }
}

#[cfg(test)]
mod tests {
    use crate::source_mapper::MappedLine;

    use super::SourceMapper;

    #[test]
    fn it_works() {
        let mut mapper = SourceMapper::default();
        let id = mapper.add("boop.txt".into(), "hi\nthere".into());
        assert_eq!(mapper.get_contents(id), "hi\nthere");
        assert_eq!(
            mapper.get_first_line(&(0, 1, Some(id))),
            Some(MappedLine::new(0, 0, 1, "hi"))
        );
        assert_eq!(
            mapper.get_first_line(&(3, 4, Some(id))),
            Some(MappedLine::new(1, 0, 1, "there"))
        );
        assert_eq!(
            mapper.get_first_line(&(0, 4, Some(id))),
            Some(MappedLine::new(0, 0, 2, "hi"))
        );
    }
}
