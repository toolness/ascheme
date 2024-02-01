use parser::Expression;

use crate::{
    environment::Environment,
    interpreter::Interpreter,
    parser::{parse, ExpressionValue},
    source_mapper::SourceMapper,
    string_interner::StringInterner,
};

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
    let mut mapper = SourceMapper::default();
    let source_id = mapper.add("<String>".into(), "  (+ 1 2 (* 3 4)) ".into());
    let mut interner = StringInterner::default();
    let mut environment = Environment::default();
    builtins::populate_environment(&mut environment, &mut interner);
    let parsed = parse(
        mapper.get_contents(source_id),
        &mut interner,
        Some(source_id),
    )
    .unwrap();
    let mut interpreter = Interpreter::new(environment).with_source_mapper(mapper);

    println!("{}", stringify_expressions(&parsed, &interner));
    println!(
        "Evaluation result: {:?}",
        interpreter.eval_expressions(&parsed)
    );
}
