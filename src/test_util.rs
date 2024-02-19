use crate::{interpreter::Interpreter, value::Value};

pub fn test_eval_successes(code_and_expected_values: &[(&str, &str)]) {
    let mut interpreter = Interpreter::new();
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
                assert_eq!(value, expected_value, "Evaluating code #{i} '{code}'");
            }
            Err(err) => {
                panic!("Evaluating code #{i} '{code}' raised error {err:?}");
            }
        }
    }
}

pub fn test_eval_success(code: &'static str, expected_value: &'static str) {
    test_eval_successes(&[(code, expected_value)]);
}
