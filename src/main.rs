use crate::{string_interner::StringInterner, tokenizer::Tokenizer};

mod string_interner;
mod tokenizer;

fn main() {
    let mut k = Tokenizer::new(&"  (  ) ");
    let mut interner = StringInterner::default();

    let boop1 = interner.intern("boop");
    let boop2 = interner.intern("boop");
    let bap = interner.intern("bap");

    println!(
        "Hello, world! {:?} {:?} {boop1:?} {boop2:?} {bap:?} {}",
        k.next(),
        k.next(),
        interner.get(boop1)
    );
}
