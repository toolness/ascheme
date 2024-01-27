use crate::{
    parser::{Expression, Parser},
    string_interner::StringInterner,
    tokenizer::Tokenizer,
};

mod parser;
mod string_interner;
mod tokenizer;

fn main() {
    let code = "  (+ 1 2 (* 3 4)) ";
    let mut k = Tokenizer::new(&code);
    let mut interner = StringInterner::default();
    let parser = Parser::new(code, Tokenizer::new(&code), &mut interner);
    let parsed = parser.collect::<Result<Vec<Expression>, _>>();

    let boop1 = interner.intern("boop");
    let boop2 = interner.intern("boop");
    let bap = interner.intern("bap");

    println!(
        "Hello, world! {:?} {:?} {boop1:?} {boop2:?} {bap:?} {} {parsed:?}",
        k.next(),
        k.next(),
        interner.get(boop1)
    );
}
