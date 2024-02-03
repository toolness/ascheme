use std::{fs::read_to_string, process};

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
    match interpreter.evaluate(source_id) {
        Ok(value) => println!("{:?}", value),
        Err(err) => {
            if let Some(trace) = interpreter.source_mapper.trace(&err.1) {
                println!("Error: {:?} in {}", err.0, trace.join("\n"));
            } else {
                println!("Error: {:?}", err.0);
            }
            println!("{}", interpreter.traceback());
            process::exit(1)
        }
    }
}
