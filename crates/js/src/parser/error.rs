use sl_std::slice::SubsliceOffset;

/// A pointer the the location of a syntax error in a script
///
/// The offset is in bytes
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SyntaxError {
    position: usize,
    message: String,
}

impl SyntaxError {
    #[must_use]
    pub const fn new(position: usize, message: String) -> Self {
        Self { position, message }
    }

    pub fn get_context<'source_code, 'error>(
        &'error self,
        context: &'source_code str,
    ) -> ErrorContext<'source_code, 'error> {
        for line in context.lines() {
            let byte_range = context
                .subslice_range(line)
                .expect("Line is not a reference to the source string");

            if byte_range.contains(self.position) {
                return ErrorContext {
                    line,
                    offset_in_line: self.position - byte_range.start(),
                    message: &self.message,
                };
            }
        }

        // If we reach this point then the syntax error must be at the end of the input
        let last_line = context.lines().last().unwrap_or_default();
        ErrorContext {
            line: last_line,
            offset_in_line: last_line.len(),
            message: &self.message,
        }
    }
}

/// A [SyntaxError] with added source code annotations for more context
#[derive(Clone, Copy, Debug)]
pub struct ErrorContext<'source_code, 'error> {
    pub line: &'source_code str,
    pub offset_in_line: usize,
    pub message: &'error str,
}

impl<'source_code, 'error> ErrorContext<'source_code, 'error> {
    pub fn dump(&self) {
        println!("Error Context:");
        println!("* {}", self.line);
        println!("* {}^ {}", " ".repeat(self.offset_in_line), self.message)
    }
}
