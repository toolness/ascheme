use std::{env::args, fs::read_to_string, process};

use crate::interpreter::Interpreter;

mod builtins;
mod compound_procedure;
mod environment;
mod interpreter;
mod parser;
mod source_mapped;
mod source_mapper;
mod string_interner;
mod tokenizer;

fn main() {
    if let Some(filename) = args().nth(1) {
        let contents = read_to_string(&filename).unwrap();
        let mut interpreter = Interpreter::new();
        let source_id = interpreter.source_mapper.add(filename, contents);
        println!("{:?}", interpreter.evaluate(source_id));
    } else {
        eprintln!("Please specify a filename.");
        process::exit(1);
    }
}
