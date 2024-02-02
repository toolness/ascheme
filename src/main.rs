use parser::Expression;

use crate::{interpreter::Interpreter, parser::ExpressionValue, string_interner::StringInterner};

mod builtins;
mod compound_procedure;
mod environment;
mod interpreter;
mod parser;
mod source_mapped;
mod source_mapper;
mod string_interner;
mod tokenizer;

fn stringify_expressions(expressions: &Vec<Expression>, interner: &StringInterner) -> String {
    let mut items: Vec<String> = vec![];
    for expression in expressions {
        let item_string = match &expression.0 {
            ExpressionValue::Number(num) => num.to_string(),
            ExpressionValue::Symbol(string) => interner.get(string).to_string(),
            ExpressionValue::Combination(values) => {
                format!("({})", stringify_expressions(values, interner))
            }
        };
        items.push(item_string);
    }
    items.join(" ")
}

fn main() {
    let mut interpreter = Interpreter::new();
    let source_id = interpreter
        .source_mapper
        .add("<String>".into(), "  (+ 1 2 (* 3 4)) ".into());

    println!(
        "{}",
        stringify_expressions(
            &interpreter.parse(source_id).unwrap(),
            &interpreter.string_interner
        )
    );
    println!("Evaluation result: {:?}", interpreter.evaluate(source_id));
}
