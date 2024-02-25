use std::{cell::RefCell, fmt::Display};

/// If we don't get a newline for these many characters, flush the output
/// to stdout.
const MAX_BUFFER_SIZE: usize = 255;

/// This is a weird class that buffers lines internally, which gives us
/// control over how we output buffered data.  We need it in part because
/// rustyline appears to overwrite any content on the current line that
/// it's prompting, which requires us to pass buffered output to it
/// so prompts work as expected when running programs.
pub struct StdioPrinter {
    pub disable_autoflush: bool,
    line_buffer: RefCell<String>,
}

impl StdioPrinter {
    pub fn new() -> Self {
        StdioPrinter {
            disable_autoflush: false,
            line_buffer: String::with_capacity(MAX_BUFFER_SIZE).into(),
        }
    }

    #[cfg(test)]
    pub fn take_buffered_output(&self) -> String {
        self.line_buffer.take()
    }

    fn flush_line_buffer(&self) {
        print!("{}", self.line_buffer.borrow());
        self.line_buffer.borrow_mut().clear();
    }

    /// Print out any buffered output followed by a newline.
    pub fn print_buffered_output(&self) {
        if !self.line_buffer.borrow().is_empty() {
            self.line_buffer.borrow_mut().push('\n');
            self.flush_line_buffer();
        }
    }

    /// Print the given string to stdout in a line-buffered way.
    pub fn print<T: AsRef<str>>(&self, value: T) {
        for ch in value.as_ref().chars() {
            self.line_buffer.borrow_mut().push(ch);

            if !self.disable_autoflush && ch == '\n'
                || self.line_buffer.borrow().len() == MAX_BUFFER_SIZE
            {
                self.flush_line_buffer();
            }
        }
    }

    /// Print the given string to stdout in a line-buffered way, followed by a newline.
    pub fn println<T: AsRef<str>>(&self, value: T) {
        self.print(value);
        self.print("\n");
    }

    /// Print any buffered output, then write the given string to stderr
    /// followed by a newline.
    ///
    /// This ensures that users see any partially printed output before
    /// error output in programs.
    pub fn eprintln<T: Display>(&self, value: T) {
        self.print_buffered_output();
        eprintln!("{}", value);
    }
}
