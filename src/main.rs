use std::fs::read_to_string;

use clap::Parser;

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

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    /// Source file to execute.
    pub source_filename: String,

    /// Enable source code tracing
    #[arg(short, long)]
    pub tracing: bool,
}

fn main() {
    let args = CliArgs::parse();

    let contents = read_to_string(&args.source_filename).unwrap();
    let mut interpreter = Interpreter::new();
    interpreter.tracing = args.tracing;
    let source_id = interpreter
        .source_mapper
        .add(args.source_filename.clone(), contents);
    println!("{:?}", interpreter.evaluate(source_id));
}
