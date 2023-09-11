use std::fmt;

const WHITESPACE_PER_INDENT_LEVEL: usize = 2;
const MAX_TEXT_LEN: usize = 16;

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

    pub fn indent(&mut self) -> fmt::Result {
        write!(
            self.formatter,
            "{}",
            " ".repeat(self.indent_level * WHITESPACE_PER_INDENT_LEVEL)
        )
    }

    pub fn write_text(&mut self, text: &str) -> fmt::Result {
        // Newlines in the text will mess up the tree
        let escaped: String = text.escape_default().collect();

        if escaped.len() < MAX_TEXT_LEN {
            write!(self.formatter, "{escaped:?}",)
        } else {
            write!(self.formatter, "\"{} [...]\"", &escaped[..MAX_TEXT_LEN])
        }
    }
}

impl<'a, 'b> fmt::Write for TreeFormatter<'a, 'b> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.formatter.write_str(s)
    }
}

pub trait TreeDebug {
    fn tree_fmt(&self, formatter: &mut TreeFormatter<'_, '_>) -> fmt::Result;
}
