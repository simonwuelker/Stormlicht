use sl_std::chars::ReversibleCharIterator;

use crate::{Deserialize, Deserializer, Map, Sequence};

#[derive(Clone, Debug)]
pub enum JsonError {
    UnexpectedToken,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Token {
    String(String),
    Numeric(u32),
    True,
    False,
    Null,
    CurlyBraceOpen,
    CurlyBraceClose,
    BracketOpen,
    BracketClose,
    Colon,
    Comma,
}

pub struct JsonDeserializer<'a> {
    chars: ReversibleCharIterator<&'a str>,
}

impl<'a> JsonDeserializer<'a> {
    pub fn new(json: &'a str) -> Self {
        Self {
            chars: ReversibleCharIterator::new(json),
        }
    }

    fn expect_next_token(&mut self, expected_token: Token) -> Result<(), JsonError> {
        if self.next_token() == Some(expected_token) {
            Ok(())
        } else {
            Err(JsonError::UnexpectedToken)
        }
    }

    fn deserialize_optional<T: Deserialize>(&mut self) -> Option<T> {
        let old_pos = self.chars.position();
        if let Ok(value) = T::deserialize(self) {
            Some(value)
        } else {
            self.chars.set_position(old_pos);
            None
        }
    }

    fn consume_hex_escape(&mut self) -> Option<u32> {
        // Expect 4 hex digits
        let a = self.chars.next()?;
        let b = self.chars.next()?;
        let c = self.chars.next()?;
        let d = self.chars.next()?;

        let value = (a.to_digit(16)?) << 12
            | (b.to_digit(16)?) << 8
            | (c.to_digit(16)?) << 4
            | (d.to_digit(16)?);
        Some(value)
    }

    fn next_token(&mut self) -> Option<Token> {
        loop {
            match self.chars.current() {
                Some('{') => {
                    self.chars.next();
                    return Some(Token::CurlyBraceOpen);
                },
                Some('}') => {
                    self.chars.next();
                    return Some(Token::CurlyBraceClose);
                },
                Some('[') => {
                    self.chars.next();
                    return Some(Token::BracketOpen);
                },
                Some(']') => {
                    self.chars.next();
                    return Some(Token::BracketClose);
                },
                Some(',') => {
                    self.chars.next();
                    return Some(Token::Comma);
                },
                Some(':') => {
                    self.chars.next();
                    return Some(Token::Colon);
                },
                Some('"') => {
                    // Parse a string
                    let mut string = String::new();
                    self.chars.next();
                    while let Some(c) = self.chars.current() {
                        self.chars.next();
                        match c {
                            '"' => {
                                return Some(Token::String(string));
                            },
                            '\\' => {
                                match self.chars.next()? {
                                    escaped_char @ ('\\' | '"' | '/') => string.push(escaped_char),
                                    'b' => string.push('\x08'),
                                    'f' => string.push('\x0c'),
                                    'n' => string.push('\n'),
                                    'r' => string.push('\r'),
                                    't' => string.push('\t'),
                                    'u' => {
                                        let mut reference_value = self.consume_hex_escape()?;
                                        if (0xD800..=0xDBFF).contains(&reference_value) {
                                            // UTF-16 surrogate
                                            if self.chars.next() != Some('\\') {
                                                return None;
                                            }
                                            if self.chars.next() != Some('u') {
                                                return None;
                                            }
                                            let second_reference = self.consume_hex_escape()?;

                                            if !(0xDC00..=0xDFFF).contains(&second_reference) {
                                                return None;
                                            }

                                            reference_value = ((reference_value - 0xD800) << 10
                                                | (second_reference - 0xDC00))
                                                + 0x1_0000;
                                        }

                                        let referenced_char = char::from_u32(reference_value)?;
                                        string.push(referenced_char);
                                    },
                                    _ => {
                                        // Invalid escape character
                                        return None;
                                    },
                                }
                            },
                            other => {
                                string.push(other);
                            },
                        }
                    }

                    // EOF in string
                    return None;
                },
                Some(' ' | '\t' | '\r' | '\n') => {
                    // whitespace is skipped
                    self.chars.next();
                },
                Some('0'..='9') => {
                    // Parse a numeric value
                    let mut num = 0;

                    while let Some(c) = self.chars.current() {
                        if let Some(digit) = c.to_digit(10) {
                            num *= 10;
                            num += digit;
                            self.chars.next();
                        } else {
                            break;
                        }
                    }

                    return Some(Token::Numeric(num));
                },
                Some('t' | 'f' | 'n') => {
                    // Parse an identifier (true, false or null)
                    let remaining = self.chars.remaining();
                    return if remaining.starts_with("true") {
                        _ = self.chars.advance_by("true".len());
                        Some(Token::True)
                    } else if remaining.starts_with("false") {
                        _ = self.chars.advance_by("false".len());
                        Some(Token::False)
                    } else if remaining.starts_with("null") {
                        _ = self.chars.advance_by("null".len());
                        Some(Token::Null)
                    } else {
                        None
                    };
                },
                Some(_) | None => return None,
            }
        }
    }

    fn peek_token(&mut self) -> Option<Token> {
        let old_position = self.chars.position();
        let token = self.next_token();
        self.chars.set_position(old_position);
        token
    }
}

impl<'a> Deserializer for JsonDeserializer<'a> {
    type Error = JsonError;

    fn start_struct(&mut self) -> Result<(), Self::Error> {
        self.expect_next_token(Token::CurlyBraceOpen)
    }

    fn end_struct(&mut self) -> Result<(), Self::Error> {
        self.expect_next_token(Token::CurlyBraceClose)
    }

    fn deserialize_field<T: Deserialize>(&mut self, name: &str) -> Result<T, Self::Error> {
        self.expect_next_token(Token::String(name.to_string()))?;
        self.expect_next_token(Token::Colon)?;
        let value = T::deserialize(self)?;

        // Not entirely json compliant: the comma is always optional
        let old_position = self.chars.position();
        if self.next_token() != Some(Token::Comma) {
            self.chars.set_position(old_position);
        }
        Ok(value)
    }

    fn deserialize_sequence<S: Sequence>(&mut self) -> Result<S, Self::Error>
    where
        S::Item: Deserialize,
    {
        self.expect_next_token(Token::BracketOpen)?;
        let mut values = S::default();
        if let Some(value) = self.deserialize_optional() {
            values.add_item(value);
        }
        while self.peek_token() == Some(Token::Comma) {
            self.next_token(); // discard the comma

            values.add_item(S::Item::deserialize(self)?);
        }
        self.expect_next_token(Token::BracketClose)?;
        Ok(values)
    }

    fn deserialize_map<M: Map>(&mut self) -> Result<M, Self::Error>
    where
        M::Value: Deserialize,
    {
        self.start_struct()?;

        let mut map = M::default();
        if let Ok(key) = self.deserialize_string() {
            self.expect_next_token(Token::Colon)?;
            let value = M::Value::deserialize(self)?;
            map.add_key_value(key, value);
        }

        while self.peek_token() == Some(Token::Comma) {
            _ = self.next_token(); // discard the comma

            let key = self.deserialize_string()?;
            self.expect_next_token(Token::Colon)?;
            let value = M::Value::deserialize(self)?;
            map.add_key_value(key.clone(), value);
        }

        self.end_struct()?;

        Ok(map)
    }

    fn deserialize_string(&mut self) -> Result<String, Self::Error> {
        if let Some(Token::String(s)) = self.next_token() {
            Ok(s)
        } else {
            Err(JsonError::UnexpectedToken)
        }
    }

    fn deserialize_usize(&mut self) -> Result<usize, Self::Error> {
        if let Some(Token::Numeric(num)) = self.next_token() {
            Ok(num as usize)
        } else {
            Err(JsonError::UnexpectedToken)
        }
    }
}

#[cfg(test)]
mod tests {

    use super::{JsonDeserializer, Token};

    #[test]
    fn tokenize() {
        let json = "{ \"foo\": true}";
        let mut tokens = JsonDeserializer::new(json);
        assert_eq!(tokens.next_token(), Some(Token::CurlyBraceOpen));
        assert_eq!(tokens.next_token(), Some(Token::String("foo".to_string())));
        assert_eq!(tokens.next_token(), Some(Token::Colon));
        assert_eq!(tokens.next_token(), Some(Token::True));
        assert_eq!(tokens.next_token(), Some(Token::CurlyBraceClose));
        assert_eq!(tokens.next_token(), None);
    }

    #[test]
    fn unicode() {
        // <UNICODE_UMBRELLA> + newline
        let json = "\"\\u2602\\n\"";
        let mut tokens = JsonDeserializer::new(json);
        assert_eq!(tokens.next_token(), Some(Token::String("☂\n".to_string())));
        assert_eq!(tokens.next_token(), None);
    }
}
