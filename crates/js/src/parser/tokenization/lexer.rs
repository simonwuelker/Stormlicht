use sl_std::chars::ReversibleCharIterator;

use crate::parser::SyntaxError;

use super::{Punctuator, Token};

#[derive(Clone, Copy)]
pub struct Lexer<'a> {
    source: ReversibleCharIterator<&'a str>,
}

/// Check if a character is an [ECMAScript line terminator](https://262.ecma-international.org/14.0/#sec-line-terminators)
///
/// The full list of line-terminating characters can be found [here](https://262.ecma-international.org/14.0/#table-line-terminator-code-points)
#[inline]
#[must_use]
const fn is_line_terminator(c: char) -> bool {
    matches!(c, '\u{000A}' | '\u{000D}' | '\u{2028}' | '\u{2029}')
}

impl<'a> Lexer<'a> {
    #[must_use]
    pub const fn new(source_text: &'a str) -> Self {
        Self {
            source: ReversibleCharIterator::new(source_text),
        }
    }

    #[must_use]
    pub fn syntax_error(&self) -> SyntaxError {
        SyntaxError::from_position(self.source.position())
    }

    pub fn next_token(&mut self) -> Result<Option<Token>, SyntaxError> {
        // Skip any leading whitespace characters
        let current_char = loop {
            let Some(current_char) = self.source.current() else {
                // At end of input
                return Ok(None);
            };

            // FIXME: This is not correct, we should skip characters as listed in
            //        https://262.ecma-international.org/14.0/#table-white-space-code-points
            if !current_char.is_whitespace() {
                break current_char;
            }
            self.source.next();
        };

        let token = match current_char {
            c if is_line_terminator(c) => {
                self.source.next();
                Token::LineTerminator
            },
            '#' => {
                self.source.next();

                let private_identifier = self.consume_identifier()?;
                Token::PrivateIdentifier(private_identifier)
            },
            '0'..='9' => {
                let numeric_literal = self.consume_numeric_literal()?;
                Token::NumericLiteral(numeric_literal)
            },
            '"' | '\'' => {
                let string_literal = self.consume_string_literal()?;
                Token::StringLiteral(string_literal)
            },
            _ => {
                if self.attempt(Self::consume_null_literal).is_ok() {
                    Token::NullLiteral
                } else if let Ok(boolean_literal) = self.attempt(Self::consume_boolean_literal) {
                    Token::BooleanLiteral(boolean_literal)
                } else if let Ok(identifier) = self.attempt(Self::consume_identifier) {
                    Token::Identifier(identifier)
                } else if let Ok(punctuator) = self.attempt(Self::consume_punctuator) {
                    Token::Punctuator(punctuator)
                } else {
                    // Cannot parse input stream as a valid token
                    return Err(self.syntax_error());
                }
            },
        };

        Ok(Some(token))
    }

    pub fn attempt<F, T, E>(&mut self, f: F) -> Result<T, E>
    where
        F: FnOnce(&mut Self) -> Result<T, E>,
    {
        let position = self.source.position();

        let parse_result = f(self);

        if parse_result.is_err() {
            self.source.set_position(position);
        }

        parse_result
    }

    fn skip_whitespace(&mut self) {
        // FIXME: This is not correct, we should skip characters as listed in
        //        https://262.ecma-international.org/14.0/#table-white-space-code-points
        while self.source.next().is_some_and(char::is_whitespace) {}
        self.source.go_back()
    }

    pub fn expect_keyword(&mut self, keyword: &str) -> Result<(), SyntaxError> {
        if self.source.remaining().starts_with(keyword) {
            _ = self.source.advance_by(keyword.len());
            self.skip_whitespace();
            Ok(())
        } else {
            Err(self.syntax_error())
        }
    }

    pub fn consume_null_literal(&mut self) -> Result<(), SyntaxError> {
        self.expect_keyword("null")
    }

    pub fn consume_boolean_literal(&mut self) -> Result<bool, SyntaxError> {
        let remaining = self.source.remaining();
        let boolean_literal = if remaining.starts_with("true") {
            _ = self.source.advance_by("true".len());
            true
        } else if remaining.starts_with("false") {
            _ = self.source.advance_by("false".len());
            false
        } else {
            return Err(self.syntax_error());
        };

        self.skip_whitespace();
        Ok(boolean_literal)
    }

    /// <https://262.ecma-international.org/14.0/#prod-IdentifierName>
    pub fn consume_identifier(&mut self) -> Result<String, SyntaxError> {
        let c = self.source.next().ok_or(self.syntax_error())?;

        if !is_valid_identifier_start(c) {
            return Err(self.syntax_error());
        }

        let mut identifier = c.to_string();

        while let Some(c) = self.source.next() {
            // FIXME: this is the IdentifierPart production and its not right
            if !c.is_ascii_alphabetic() {
                self.source.go_back();
                break;
            }

            identifier.push(c);
        }

        self.skip_whitespace();
        Ok(identifier)
    }

    /// <https://262.ecma-international.org/14.0/#prod-StringLiteral>
    pub fn consume_string_literal(&mut self) -> Result<String, SyntaxError> {
        let consume_string_literal_until =
            |tokenizer: &mut Lexer<'_>, terminator| -> Result<String, SyntaxError> {
                // FIXME: this isn't correct
                let mut literal = String::new();
                for c in tokenizer.source.by_ref() {
                    if c == terminator {
                        return Ok(literal);
                    } else {
                        literal.push(c);
                    }
                }
                // Unterminated string literal
                Err(tokenizer.syntax_error())
            };

        let string_literal = match self.source.next() {
            Some(c @ ('"' | '\'')) => consume_string_literal_until(self, c)?,
            _ => return Err(self.syntax_error()),
        };

        self.skip_whitespace();
        Ok(string_literal)
    }

    /// <https://262.ecma-international.org/14.0/#prod-NumericLiteral>
    pub fn consume_numeric_literal(&mut self) -> Result<u32, SyntaxError> {
        match self.source.next() {
            Some('0') => Ok(0),
            Some(c @ ('1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9')) => {
                let mut n = c.to_digit(10).expect("Characters 1-9 are decimal digits");
                loop {
                    match self.source.next() {
                        Some('_') => continue,
                        Some(c) => {
                            if let Some(digit) = c.to_digit(10) {
                                n *= 10;
                                n += digit;
                            } else {
                                self.source.go_back();
                                break;
                            }
                        },
                        _ => {
                            self.source.go_back();
                            break;
                        },
                    }
                }

                Ok(n)
            },
            _ => Err(self.syntax_error()),
        }
    }

    /// <https://262.ecma-international.org/14.0/#sec-punctuators>
    pub fn consume_punctuator(&mut self) -> Result<Punctuator, SyntaxError> {
        let punctuator = match self.source.next() {
            Some('?') => match self.source.current() {
                Some('?') => {
                    self.source.next();
                    if self.source.current() == Some('=') {
                        self.source.next();
                        Punctuator::DoubleQuestionMarkEqual
                    } else {
                        Punctuator::DoubleQuestionMark
                    }
                },
                Some('.') => {
                    self.source.next();
                    if self.source.current().is_some_and(|c| c.is_ascii_digit()) {
                        // Lookahead failed
                        self.source.go_back();
                        Punctuator::QuestionMark
                    } else {
                        Punctuator::OptionalChaining
                    }
                },
                _ => Punctuator::QuestionMark,
            },
            Some('{') => Punctuator::CurlyBraceOpen,
            Some('(') => Punctuator::ParenthesisOpen,
            Some(')') => Punctuator::ParenthesisClose,
            Some('[') => Punctuator::BracketOpen,
            Some(']') => Punctuator::BracketClose,
            Some('.') => {
                if self.source.remaining().starts_with("..") {
                    _ = self.source.advance_by(2);
                    Punctuator::TripleDot
                } else {
                    Punctuator::Dot
                }
            },
            Some(';') => Punctuator::Semicolon,
            Some(',') => Punctuator::Comma,
            Some('<') => match self.source.current() {
                Some('<') => {
                    self.source.next();
                    if self.source.current() == Some('=') {
                        self.source.next();
                        Punctuator::DoubleLessThanEqual
                    } else {
                        Punctuator::DoubleLessThan
                    }
                },
                Some('=') => {
                    self.source.next();
                    Punctuator::LessThanEqual
                },
                _ => Punctuator::LessThan,
            },
            Some('>') => match self.source.current() {
                Some('>') => {
                    self.source.next();
                    match self.source.current() {
                        Some('>') => {
                            self.source.next();
                            if self.source.current() == Some('=') {
                                self.source.next();
                                Punctuator::TripleGreaterThanEqual
                            } else {
                                Punctuator::TripleGreaterThan
                            }
                        },
                        Some('=') => {
                            self.source.next();
                            Punctuator::DoubleGreaterThanEqual
                        },
                        _ => Punctuator::DoubleGreaterThan,
                    }
                },
                Some('=') => {
                    self.source.next();
                    Punctuator::GreaterThanEqual
                },
                _ => Punctuator::GreaterThan,
            },
            Some('=') => {
                if self.source.current() == Some('=') {
                    self.source.next();
                    if self.source.current() == Some('=') {
                        self.source.next();
                        Punctuator::TripleEqual
                    } else {
                        Punctuator::DoubleEqual
                    }
                } else {
                    Punctuator::Equal
                }
            },
            Some('!') => {
                if self.source.current() == Some('=') {
                    self.source.next();
                    if self.source.current() == Some('=') {
                        self.source.next();
                        Punctuator::ExclamationMarkDoubleEqual
                    } else {
                        Punctuator::ExclamationMarkEqual
                    }
                } else {
                    Punctuator::ExclamationMark
                }
            },
            Some('+') => match self.source.current() {
                Some('+') => {
                    self.source.next();
                    Punctuator::DoublePlus
                },
                Some('=') => {
                    self.source.next();
                    Punctuator::PlusEqual
                },
                _ => Punctuator::Plus,
            },
            Some('-') => match self.source.current() {
                Some('-') => {
                    self.source.next();
                    Punctuator::DoubleMinus
                },
                Some('=') => {
                    self.source.next();
                    Punctuator::MinusEqual
                },
                _ => Punctuator::Minus,
            },
            Some('*') => match self.source.current() {
                Some('*') => {
                    self.source.next();
                    if self.source.current() == Some('=') {
                        self.source.next();
                        Punctuator::DoubleAsteriskEqual
                    } else {
                        Punctuator::DoubleAsterisk
                    }
                },
                Some('=') => {
                    self.source.next();
                    Punctuator::AsteriskEqual
                },
                _ => Punctuator::Asterisk,
            },
            Some('%') => {
                if self.source.current() == Some('=') {
                    self.source.next();
                    Punctuator::PercentEqual
                } else {
                    Punctuator::Percent
                }
            },
            Some('&') => match self.source.current() {
                Some('&') => {
                    self.source.next();
                    if self.source.current() == Some('=') {
                        self.source.next();
                        Punctuator::DoubleAmpersandEqual
                    } else {
                        Punctuator::DoubleAmpersand
                    }
                },
                Some('=') => {
                    self.source.next();
                    Punctuator::AmpersandEqual
                },
                _ => Punctuator::Ampersand,
            },
            Some('|') => match self.source.current() {
                Some('|') => {
                    self.source.next();
                    if self.source.current() == Some('=') {
                        self.source.next();
                        Punctuator::DoubleVerticalBarEqual
                    } else {
                        Punctuator::DoubleVerticalBar
                    }
                },
                Some('=') => {
                    self.source.next();
                    Punctuator::VerticalBarEqual
                },
                _ => Punctuator::VerticalBar,
            },
            Some('^') => {
                if self.source.current() == Some('=') {
                    self.source.next();
                    Punctuator::CaretEqual
                } else {
                    Punctuator::Caret
                }
            },
            Some('~') => Punctuator::Tilde,
            Some(':') => Punctuator::Colon,
            Some('}') => Punctuator::CurlyBraceClose,
            _ => return Err(self.syntax_error()),
        };

        self.skip_whitespace();
        Ok(punctuator)
    }
}

#[inline]
#[must_use]
/// Check if a character can be the start of an [ECMAScript identifier](https://262.ecma-international.org/14.0/#prod-IdentifierStart)
fn is_valid_identifier_start(c: char) -> bool {
    // FIXME: this is the IdentifierStart production and its not right
    matches!(c, '$' | '_' | 'a'..='z' | 'A'..='Z')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_punctuator() {
        assert_eq!(
            Lexer::new("?.").consume_punctuator(),
            Ok(Punctuator::OptionalChaining)
        );

        assert_ne!(
            Lexer::new("?.5").consume_punctuator(),
            Ok(Punctuator::OptionalChaining)
        );
    }

    #[test]
    fn tokenize_string_literal() {
        assert_eq!(
            Lexer::new("\"foobar\"").consume_string_literal().as_deref(),
            Ok("foobar")
        );

        assert_eq!(
            Lexer::new("'foobar'").consume_string_literal().as_deref(),
            Ok("foobar")
        );
    }
}
