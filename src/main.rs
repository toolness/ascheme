use std::sync::mpsc::channel;
use std::{fs::read_to_string, process};

use clap::Parser;
use ctrlc;
use parser::{parse, ParseErrorType};
use rustyline::{Completer, Editor, Helper, Highlighter, Hinter};
use source_mapper::SourceId;
use string_interner::StringInterner;
use value::Value;

use crate::interpreter::Interpreter;

use rustyline::error::ReadlineError;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};

mod builtins;
mod compound_procedure;
mod environment;
mod interpreter;
mod pair;
mod parser;
mod source_mapped;
mod source_mapper;
mod string_interner;
mod tokenizer;
mod value;

#[cfg(test)]
mod interpreter_tests;

const HISTORY_FILENAME: &'static str = ".interpreter-history.txt";

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    /// Source file to execute.
    pub source_filename: Option<String>,

    /// Enable source code tracing
    #[arg(short, long)]
    pub tracing: bool,

    /// Continue in interactive mode after executing source file.
    #[arg(short, long)]
    pub interactive: bool,
}

#[derive(Completer, Helper, Highlighter, Hinter)]
struct SchemeInputValidator();

impl Validator for SchemeInputValidator {
    fn validate(&self, ctx: &mut ValidationContext<'_>) -> rustyline::Result<ValidationResult> {
        let input = ctx.input();
        let mut interner = StringInterner::default();
        let Err(err) = parse(input, &mut interner, None) else {
            return Ok(ValidationResult::Valid(None));
        };

        match err.0 {
            ParseErrorType::MissingRightParen => Ok(ValidationResult::Incomplete),
            // There's an error, but the interpreter will show it to the user--we just want to let
            // rustyline know whether to let the user continue typing.
            _ => Ok(ValidationResult::Valid(None)),
        }
    }
}

/// Returns true on success, false on failure.
fn evaluate(interpreter: &mut Interpreter, source_id: SourceId) -> bool {
    match interpreter.evaluate(source_id) {
        Ok(value) => {
            if value.0 != Value::Undefined {
                println!("{}", value);
            }
            true
        }
        Err(err) => {
            println!(
                "Error: {:?} in {}",
                err.0,
                interpreter.source_mapper.trace(&err.1).join("\n")
            );
            println!("{}", interpreter.traceback());
            false
        }
    }
}

fn main() {
    let args = CliArgs::parse();
    let (tx, rx) = channel();

    ctrlc::set_handler(move || tx.send(()).expect("Count not send signal on channel."))
        .expect("Error setting Ctrl-C handler.");

    let mut interpreter = Interpreter::new();
    interpreter.tracing = args.tracing;
    interpreter.keyboard_interrupt_channel = Some(rx);

    if let Some(filename) = args.source_filename {
        let contents = read_to_string(&filename).unwrap();
        let source_id = interpreter.source_mapper.add(filename, contents);
        let success = evaluate(&mut interpreter, source_id);
        if !args.interactive {
            process::exit(if success { 0 } else { 1 });
        }
    }

    let Ok(mut rl) = Editor::new() else {
        eprintln!("Initializing DefaultEditor failed!");
        process::exit(1);
    };

    rl.set_helper(Some(SchemeInputValidator()));

    // Note that we're ignoring the result here, which is generally OK--if it
    // errors, it's probably because the file doesn't exist, and even then
    // history is optional anyways.
    let _ = rl.load_history(HISTORY_FILENAME);
    let mut i = 0;

    loop {
        match rl.readline("> ") {
            Ok(line) => {
                // Again, we're ignoring the result here, see above for rationale.
                let _ = rl.add_history_entry(line.as_str());

                i += 1;
                let filename = format!("<Input#{i}>");
                let source_id = interpreter.source_mapper.add(filename, line);
                evaluate(&mut interpreter, source_id);
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C pressed, exiting.");
                break;
            }
            Err(ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                process::exit(1);
            }
        }
    }

    // Again, we're ignoring the result here, see above for rationale.
    let _ = rl.save_history(HISTORY_FILENAME);
}
