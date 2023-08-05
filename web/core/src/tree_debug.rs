use std::fmt;

const WHITESPACE_PER_INDENT_LEVEL: usize = 2;

/// Utility to debug-print tree data structures
pub struct TreeFormatter<'a, 'b> {
    indent_level: usize,
    formatter: &'a mut fmt::Formatter<'b>,
}

impl<'a, 'b> TreeFormatter<'a, 'b> {
    pub fn new(formatter: &'a mut fmt::Formatter<'b>) -> Self {
        Self {
            indent_level: 0,
            formatter,
        }
    }

    pub fn increase_indent(&mut self) {
        self.indent_level += 1;
    }

    pub fn decrease_indent(&mut self) {
        assert!(self.indent_level != 0);
        self.indent_level -= 1;
    }
}

impl<'a, 'b> fmt::Write for TreeFormatter<'a, 'b> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.formatter
            .write_str(&" ".repeat(self.indent_level * WHITESPACE_PER_INDENT_LEVEL))?;
        self.formatter.write_str(s)
    }
}

pub trait TreeDebug {
    fn tree_fmt(&self, formatter: &mut TreeFormatter<'_, '_>) -> std::fmt::Result;
}