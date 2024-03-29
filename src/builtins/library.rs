use crate::source_mapper::{SourceId, SourceMapper};

pub fn add_library_source(source_mapper: &mut SourceMapper) -> SourceId {
    let library_contents = include_str!("library.sch");
    source_mapper.add("library.sch".to_string(), library_contents.to_string())
}

#[cfg(test)]
mod tests {
    use crate::test_util::{eval_test_file, test_eval_success};

    #[test]
    fn abs_works() {
        test_eval_success("(abs 1)", "1");
        test_eval_success("(abs -1)", "1");
        test_eval_success("(abs 0)", "0");
    }

    #[test]
    fn newline_works() {
        test_eval_success("(newline)", "\n");
    }

    #[test]
    fn zero_works() {
        test_eval_success("(zero? 0)", "#t");
        test_eval_success("(zero? 0.0)", "#t");
        test_eval_success("(zero? 1)", "#f");
    }

    #[test]
    fn null_works() {
        test_eval_success("(null? 0)", "#f");
        test_eval_success("(null? '())", "#t");
    }

    #[test]
    fn test_file_works() {
        eval_test_file("src/builtins/library.test.sch");
    }
}
