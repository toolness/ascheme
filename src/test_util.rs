use std::fs::read_to_string;

use crate::{
    interpreter::{Interpreter, RuntimeErrorType},
    value::Value,
};

pub fn eval_test_file(filename: &str) {
    let mut interpreter = Interpreter::new();
    let code = read_to_string(filename).unwrap();
    let source_id = interpreter.source_mapper.add(filename.into(), code);
    match interpreter.evaluate(source_id) {
        Ok(_value) => {
            assert_eq!(
                interpreter.failed_tests, 0,
                "Evaluating '{filename}' should not fail any tests."
            );
        }
        Err(err) => {
            interpreter.show_err_and_traceback(err);
            panic!("Evaluating '{filename}' raised error");
        }
    }
}

pub fn test_eval_successes(code_and_expected_values: &[(&str, &str)]) {
    let mut interpreter = Interpreter::new();
    interpreter.printer.disable_autoflush = true;
    for (i, &(code, expected_value)) in code_and_expected_values.iter().enumerate() {
        let source_id = interpreter
            .source_mapper
            .add(format!("<code[{i}]>"), code.into());
        match interpreter.evaluate(source_id) {
            Ok(value) => {
                let value = match value.0 {
                    Value::Undefined => "".to_string(),
                    _ => value.to_string(),
                };
                let output = interpreter.printer.take_buffered_output();
                let final_value = format!("{output}{value}");
                assert_eq!(final_value, expected_value, "Evaluating code #{i} '{code}'");
            }
            Err(err) => {
                interpreter.printer.disable_autoflush = false;
                interpreter.show_err_and_traceback(err);
                panic!("Evaluating code #{i} '{code}' raised error");
            }
        }
    }
}

pub fn test_eval_success(code: &'static str, expected_value: &'static str) {
    test_eval_successes(&[(code, expected_value)]);
}

pub fn test_eval_err(code: &'static str, expected_err: RuntimeErrorType) {
    let mut interpreter = Interpreter::new();
    let source_id = interpreter.source_mapper.add("<code>".into(), code.into());
    match interpreter.evaluate(source_id) {
        Ok(value) => {
            panic!("Evaluating code '{code}' did not raise error and returned {value}");
        }
        Err(err) => {
            assert_eq!(err.0, expected_err);
        }
    }
}
