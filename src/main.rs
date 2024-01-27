use parser::Expression;

use crate::{parser::Parser, string_interner::StringInterner, tokenizer::Tokenizer};

use crate::parser::ExpressionValue;

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
    let parser = Parser::new(code, Tokenizer::new(&code), &mut interner);
    let parsed = parser.parse_all();

    println!("{}", stringify_expressions(&parsed.unwrap(), &interner));
}
