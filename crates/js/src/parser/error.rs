use sl_std::slice::SubsliceOffset;

/// A pointer the the location of a syntax error in a script
///
/// The offset is in bytes
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SyntaxError(usize);

impl SyntaxError {
    #[must_use]
    pub const fn from_position(position: usize) -> Self {
        Self(position)
    }

    pub fn get_context<'a>(&self, context: &'a str) -> ErrorContext<'a> {
        for line in context.lines() {
            let byte_range = context
                .subslice_range(line)
                .expect("Line is not a reference to the source string");
            if byte_range.contains(self.0) {
                return ErrorContext {
                    line,
                    offset_in_line: self.0 - byte_range.start(),
                };
            }
        }

        // If we reach this point then no line in the context
        // contains the syntax error
        panic!("Context does not contain source reference")
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ErrorContext<'a> {
    pub line: &'a str,
    pub offset_in_line: usize,
}

impl<'a> ErrorContext<'a> {
    pub fn dump(&self) {
        println!("{:?}", self.line);
        println!("{}^", " ".repeat(self.offset_in_line))
    }
}
