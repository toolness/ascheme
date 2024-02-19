use crate::test_util::test_eval_success;

#[test]
fn trivial_expressions_work() {
    test_eval_success("5", "5");
}

#[test]
fn dot_works() {
    test_eval_success("(quote (1 . ()))", "(1)");
    test_eval_success("(quote (1 . (2 . (3 . ()))))", "(1 2 3)");
    test_eval_success("(quote (1 . 2))", "(1 . 2)");
    test_eval_success("(quote (1 2 . 3))", "(1 2 . 3)");
}

#[test]
fn booleans_work() {
    test_eval_success("#t", "#t");
    test_eval_success("#f", "#f");
}

#[test]
fn cyclic_lists_work() {
    // TODO: Eventually we should implement proper display of cyclic lists, at which point
    // the expected values will need to change.
    test_eval_success("(define x '(1 . 2)) (set-cdr! x x) x", "<CYCLIC LIST>");
    test_eval_success(
        "(define y '(1)) (define x '(1)) (set-car! y x) (set-car! x y) x",
        "<CYCLIC LIST>",
    );
}

#[test]
fn gc_finds_cycles() {
    // These print 0 because there aren't any objects trapped in cycles--regular ref-counting
    // will clean up the data.
    test_eval_success("(gc)", "0");
    test_eval_success("(define (x n) (+ n 1)) (gc)", "0");
    test_eval_success("(define (x n) (+ n 1)) (define x 0) (gc)", "0");

    // This prints 1 because an object is caught in a cycle.
    test_eval_success(
        "(define x (quote (1 . 2))) (set-cdr! x x) (define x 0) (gc)",
        "1",
    );
}

#[test]
fn gc_does_not_collect_objects_yet_to_be_evaluated() {
    test_eval_success("(define (x) 1) (gc) (x)", "1");
}

#[test]
fn strings_work() {
    test_eval_success(r#""blarg""#, r#""blarg""#);
    test_eval_success(r#""bl\narg""#, r#""bl\narg""#);
    test_eval_success(r#""bl\"arg""#, r#""bl\"arg""#);
    test_eval_success(r#""bl\\arg""#, r#""bl\\arg""#);
}

#[test]
fn undefined_stringifies() {
    test_eval_success(
        "
    (define y 1)
    (define x '(1))
    (set-car! x (set! y 2))
    x
    ",
        "(#!void)",
    )
}
