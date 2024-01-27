use parser::Expression;

use crate::{
    interpreter::Interpreter,
    parser::{parse, ExpressionValue},
    string_interner::StringInterner,
};

mod interpreter;
mod parser;
mod source_mapped;
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
    let code = "  (+ 1 2 (* 3 4)) ";
    let mut interner = StringInterner::default();
    let parsed = parse(code, &mut interner).unwrap();

    println!("{}", stringify_expressions(&parsed, &interner));
    println!(
        "Evaluation result: {:?}",
        Interpreter::evaluate(&parsed, &mut interner)
    );
}
