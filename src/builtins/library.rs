use crate::source_mapper::{SourceId, SourceMapper};

pub fn add_library_source(source_mapper: &mut SourceMapper) -> SourceId {
    let library_contents = include_str!("library.sch");
    source_mapper.add("library.sch".to_string(), library_contents.to_string())
}

#[cfg(test)]
mod tests {
    use crate::test_util::test_eval_success;

    #[test]
    fn abs_works() {
        test_eval_success("(abs 1)", "1");
        test_eval_success("(abs -1)", "1");
        test_eval_success("(abs 0)", "0");
    }
}
