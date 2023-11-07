use std::{fmt::Write, str::Chars};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnescapeStringError {
    EscapeAtEndOfString,
    InvalidEscapedChar(char),
    InvalidEscapeCode,
}

struct Unescaper<'a>(Chars<'a>);

impl<'a> Unescaper<'a> {
    fn read_charref(&mut self, length: usize) -> Result<char, UnescapeStringError> {
        let mut char_code = 0;
        for _ in 0..length {
            let c = self
                .0
                .next()
                .ok_or(UnescapeStringError::InvalidEscapeCode)?
                .to_digit(16)
                .ok_or(UnescapeStringError::InvalidEscapeCode)?;
            char_code *= 16;
            char_code += c;
        }

        let c = char::from_u32(char_code).ok_or(UnescapeStringError::InvalidEscapeCode)?;
        Ok(c)
    }
}

impl<'a> Iterator for Unescaper<'a> {
    type Item = Result<char, UnescapeStringError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|c| match c {
            '\\' => match self.0.next() {
                Some('u') => self.read_charref(4),
                Some('U') => self.read_charref(8),
                Some('x') => self.read_charref(2),
                Some('r') => Ok('\r'),
                Some('n') => Ok('\n'),
                Some('t') => Ok('\t'),
                Some('\\') => Ok('\\'),
                Some('"') => Ok('"'),
                Some('\'') => Ok('\''),
                Some(c) => Err(UnescapeStringError::InvalidEscapedChar(c)),
                None => Err(UnescapeStringError::EscapeAtEndOfString),
            },
            c => Ok(c),
        })
    }
}

pub fn unescape_str(text: &str) -> Result<String, UnescapeStringError> {
    let mut result = String::with_capacity(text.bytes().len());
    for c in Unescaper(text.chars()) {
        result.push(c?);
    }
    Ok(result)
}

/// unlike the [std version](https://doc.rust-lang.org/std/primitive.str.html#method.escape_unicode),
/// this doesn't use braces and hex codes are uppercase(`\uABCD` instead of `\u{abcd}`).
/// Ascii characters are not escaped
pub fn unicode_escape(text: &str) -> String {
    let mut result = String::new();
    for c in text.chars() {
        match c {
            '\x20'..='\x7e' => result.push(c),
            ..='\u{FFFF}' => {
                let code = c as u32;
                let _ = write!(result, "\\u{code:0>4X}");
            },
            _ => result.push(c),
        }
    }
    result
}
