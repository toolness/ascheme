use crate::tokenizer::Tokenizer;

mod tokenizer;

fn main() {
    let mut k = Tokenizer::new(&"  (  ) ");

    println!("Hello, world! {:?} {:?}", k.next(), k.next());
}
