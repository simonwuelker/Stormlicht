use std::str::Chars;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnescapeStringError {
    EscapeAtEndOfString,
    InvalidEscapedChar(char),
    InvalidEscapeCode,
}

struct Unescaper<'a>(Chars<'a>);

impl<'a> Iterator for Unescaper<'a> {
    type Item = Result<char, UnescapeStringError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|c| match c {
            '\\' => match self.0.next() {
                Some('u') => {
                    let code = [
                        self.0
                            .next()
                            .ok_or(UnescapeStringError::InvalidEscapeCode)?
                            .to_digit(16)
                            .ok_or(UnescapeStringError::InvalidEscapeCode)?,
                        self.0
                            .next()
                            .ok_or(UnescapeStringError::InvalidEscapeCode)?
                            .to_digit(16)
                            .ok_or(UnescapeStringError::InvalidEscapeCode)?,
                        self.0
                            .next()
                            .ok_or(UnescapeStringError::InvalidEscapeCode)?
                            .to_digit(16)
                            .ok_or(UnescapeStringError::InvalidEscapeCode)?,
                        self.0
                            .next()
                            .ok_or(UnescapeStringError::InvalidEscapeCode)?
                            .to_digit(16)
                            .ok_or(UnescapeStringError::InvalidEscapeCode)?,
                    ]
                    .iter()
                    .fold(0, |acc, x| acc * 16 + x);
                    char::from_u32(code).ok_or(UnescapeStringError::InvalidEscapeCode)
                },
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
                result.push_str(&format!("\\u{:04X}", code));
            },
            _ => result.push(c),
        }
    }
    result
}
