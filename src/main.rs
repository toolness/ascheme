use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::channel;
use std::{fs::read_to_string, process};

use clap::Parser;
use ctrlc;
use pair::PairManager;
use parser::{parse, ParseErrorType};
use rustyline::{Editor, Helper, Highlighter, Hinter};
use source_mapper::SourceId;
use string_interner::StringInterner;
use tokenizer::{TokenType, TokenizeErrorType, Tokenizer};
use value::Value;

use crate::interpreter::Interpreter;

use rustyline::completion::Completer;
use rustyline::error::ReadlineError;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};

mod builtins;
mod compound_procedure;
mod environment;
mod gc;
mod gc_rooted;
mod interpreter;
mod mutable_string;
mod object_tracker;
mod pair;
mod parser;
mod source_mapped;
mod source_mapper;
mod stdio_printer;
mod string_interner;
mod tokenizer;
mod value;

#[cfg(test)]
mod test_util;

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

#[derive(Helper, Highlighter, Hinter)]
struct SchemeInputValidator(Rc<RefCell<Interpreter>>);

impl Completer for SchemeInputValidator {
    type Candidate = String;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let tokenizer = Tokenizer::new(&line, None);
        for token in tokenizer {
            let Ok(token) = token else {
                continue;
            };
            if token.0 != TokenType::Identifier {
                continue;
            }
            let range = token.1;
            if range.0 <= pos && range.1 >= pos {
                let token_str = token.source(&line);
                let interpreter = self.0.borrow();
                let matches = interpreter.environment.find_global_matches(&token_str);
                return Ok((range.0, matches));
            }
        }

        Ok((0, vec![]))
    }
}

impl Validator for SchemeInputValidator {
    fn validate(&self, ctx: &mut ValidationContext<'_>) -> rustyline::Result<ValidationResult> {
        let input = ctx.input();
        let mut interner = StringInterner::default();
        let mut pair_manager = PairManager::default();
        let Err(err) = parse(input, &mut interner, &mut pair_manager, None) else {
            return Ok(ValidationResult::Valid(None));
        };

        match err.0 {
            ParseErrorType::Tokenize(TokenizeErrorType::UnterminatedString) => {
                Ok(ValidationResult::Incomplete)
            }
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
            if !matches!(value.0, Value::Undefined) {
                interpreter.printer.println(format!("{}", value));
            }
            true
        }
        Err(err) => {
            interpreter.show_err_and_traceback(err);
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
    } else {
        println!(
            "Welcome to Atul's Scheme Interpreter v{}.",
            env!("CARGO_PKG_VERSION")
        );
        println!("Press CTRL-C to exit.");
    }

    let Ok(mut rl) = Editor::new() else {
        eprintln!("Initializing DefaultEditor failed!");
        process::exit(1);
    };

    let interpreter: Rc<RefCell<Interpreter>> = RefCell::new(interpreter).into();
    rl.set_helper(Some(SchemeInputValidator(interpreter.clone())));

    // Note that we're ignoring the result here, which is generally OK--if it
    // errors, it's probably because the file doesn't exist, and even then
    // history is optional anyways.
    let _ = rl.load_history(HISTORY_FILENAME);
    let mut i = 0;

    loop {
        interpreter.borrow().printer.print_buffered_output();
        match rl.readline("> ") {
            Ok(line) => {
                // Again, we're ignoring the result here, see above for rationale.
                let _ = rl.add_history_entry(line.as_str());

                i += 1;
                let filename = format!("<Input#{i}>");
                let mut interpreter = interpreter.borrow_mut();
                let source_id = interpreter.source_mapper.add(filename, line);
                evaluate(&mut interpreter, source_id);
            }
            Err(ReadlineError::Interrupted) => {
                interpreter
                    .borrow()
                    .printer
                    .eprintln("CTRL-C pressed, exiting.");
                break;
            }
            Err(ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                interpreter
                    .borrow()
                    .printer
                    .eprintln(format!("Error: {:?}", err));
                process::exit(1);
            }
        }
    }

    // Again, we're ignoring the result here, see above for rationale.
    let _ = rl.save_history(HISTORY_FILENAME);
}
