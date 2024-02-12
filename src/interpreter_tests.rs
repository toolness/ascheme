use crate::interpreter::Interpreter;

fn test_eval_success(code: &'static str, expected_value: &'static str) {
    let mut interpreter = Interpreter::new();
    let source_id = interpreter
        .source_mapper
        .add("<String>".into(), code.into());
    match interpreter.evaluate(source_id) {
        Ok(value) => {
            assert_eq!(
                value.to_string(),
                expected_value,
                "Evaluating code '{code}'"
            );
        }
        Err(err) => {
            panic!("Evaluating code '{code}' raised error {err:?}");
        }
    }
}

#[test]
fn trivial_expressions_work() {
    test_eval_success("5", "5");
}

#[test]
fn quote_works() {
    test_eval_success("(quote 1)", "1");
    test_eval_success("(quote (1   2    3  ))", "(1 2 3)");
    test_eval_success("(quote (1 2 3 (4)))", "(1 2 3 (4))");
    test_eval_success("(quote #t)", "#t");
    test_eval_success("(quote #f)", "#f");
    test_eval_success("(quote ())", "()");
    test_eval_success("(quote blarg)", "blarg");
}

#[test]
fn set_car_works() {
    test_eval_success("(define a (quote (1 . 2))) (set-car! a 5) a", "(5 . 2)");
}

#[test]
fn set_cdr_works() {
    test_eval_success("(define a (quote (1 . 2))) (set-cdr! a 5) a", "(1 . 5)");
}

#[test]
fn dot_works() {
    test_eval_success("(quote (1 . ()))", "(1)");
    test_eval_success("(quote (1 . (2 . (3 . ()))))", "(1 2 3)");
    test_eval_success("(quote (1 . 2))", "(1 . 2)");
    test_eval_success("(quote (1 2 . 3))", "(1 2 . 3)");
}

#[test]
fn procedure_repr_works() {
    test_eval_success("(define (boop) 1) boop", "#<procedure boop #1>");
    test_eval_success("(lambda () 1)", "#<procedure #1>");
}

#[test]
fn basic_arithmetic_works() {
    // This is how try.scheme.org works, at least.
    test_eval_success("(+)", "0");
    test_eval_success("(*)", "1");

    test_eval_success("(+ 1 2)", "3");
    test_eval_success("(+ +1 2)", "3");
    test_eval_success("(+ -10 2)", "-8");
    test_eval_success("  (+ 1 2 (* 3 4)) ", "15");
}

#[test]
fn variable_definitions_work() {
    test_eval_success("(define x 3) x", "3");
    test_eval_success("(define x 3) (define y (+ x 1)) (+ x y)", "7");
}

#[test]
fn compound_procedure_definitions_work() {
    test_eval_success("(define (x) 3)", "");
    test_eval_success("(define (x) 3) (x)", "3");
    test_eval_success("(define (add-three n) (+ 3 n)) (add-three 1)", "4");
}

#[test]
fn lambda_definitions_work() {
    test_eval_success("(define x (lambda () 3))", "");
    test_eval_success("(define x (lambda () 3)) (x)", "3");
    test_eval_success("(define add-three (lambda (n) (+ 3 n))) (add-three 1)", "4");
}

#[test]
fn booleans_works() {
    test_eval_success("#t", "#t");
    test_eval_success("#f", "#f");
}

#[test]
fn less_than_works() {
    test_eval_success("(<)", "#t");
    test_eval_success("(< 1)", "#t");
    test_eval_success("(< 1 0)", "#f");
    test_eval_success("(< 0 1)", "#t");
    test_eval_success("(< 1 1)", "#f");
    test_eval_success("(< 0 1 2)", "#t");
    test_eval_success("(< 0 1 2 3 1)", "#f");
}

#[test]
fn numeric_eq_works() {
    test_eval_success("(=)", "#t");
    test_eval_success("(= 1)", "#t");
    test_eval_success("(= 1 0)", "#f");
    test_eval_success("(= 1 1)", "#t");
    test_eval_success("(= 1 1 1)", "#t");
    test_eval_success("(= 1 2 3 4)", "#f");
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
fn set_works_with_globals() {
    test_eval_success("(define x 1) (set! x 2) x", "2");
    test_eval_success("(define x 1) (set! x (+ x 1)) x", "2");
}

#[test]
fn set_works_in_closures() {
    test_eval_success(
        "
        (define (make-incrementer)
          (define n 0)
          (lambda ()
            (set! n (+ n 1))
            n
          )
        )
        (define foo (make-incrementer))
        (foo)
        ",
        "1",
    );
}

#[test]
fn if_works() {
    test_eval_success("(if #t 1)", "1");
    test_eval_success("(if #t 1 2)", "1");
    test_eval_success("(if #f 1 2)", "2");

    // R5RS section 4.1.5 says this behavior is unspecified, we'll just return undefined.
    test_eval_success("(if #f 1)", "");
}

#[test]
fn compound_procedues_prefer_argument_values_to_globals() {
    test_eval_success(
        "
        (define n 5)
        (define (add-three n) (+ 3 n))
        (+ (add-three 1) n)
    ",
        "9",
    );
}

#[test]
fn compound_procedues_use_lexical_scope() {
    test_eval_success(
        "
        (define (make-adder n)
          (define (add-n x) (+ x n))
          add-n
        )
        (define add-three (make-adder 3))
        (add-three 1)
    ",
        "4",
    );
}
