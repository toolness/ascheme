use crate::test_util::test_eval_success;

#[test]
fn abs_works() {
    test_eval_success("(abs 1)", "1");
    test_eval_success("(abs -1)", "1");
    test_eval_success("(abs 0)", "0");
}
