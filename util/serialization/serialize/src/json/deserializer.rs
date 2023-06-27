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
    /// Internally, JSON is *always* stored in UTF-8
    /// but handling the bytes individually is simpler.
    source: &'a [u8],
    pub position: usize,
}

impl<'a> JsonDeserializer<'a> {
    pub fn new(json: &'a str) -> Self {
        Self {
            source: json.as_bytes(),
            position: 0,
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
        let old_pos = self.position;
        if let Ok(value) = T::deserialize(self) {
            Some(value)
        } else {
            self.position = old_pos;
            None
        }
    }

    fn peek(&self) -> Option<u8> {
        self.source.get(self.position).copied()
    }

    fn next(&mut self) -> Option<u8> {
        let c = self.source.get(self.position).copied();
        self.position += 1;
        c
    }

    fn bump(&mut self) {
        self.position += 1;
    }

    fn consume_hex_escape(&mut self) -> Option<u32> {
        // Expect 4 hex digits
        let a = self.next()?;
        let b = self.next()?;
        let c = self.next()?;
        let d = self.next()?;

        let to_digit = |c| match c {
            b'0'..=b'9' => Some(c - b'0'),
            b'a'..=b'z' => Some(c - b'a' + 10),
            b'A'..=b'Z' => Some(c - b'A' + 10),
            _ => None,
        };
        let value = (to_digit(a)? as u32) << 12
            | (to_digit(b)? as u32) << 8
            | (to_digit(c)? as u32) << 4
            | (to_digit(d)? as u32);
        Some(value)
    }

    fn next_token(&mut self) -> Option<Token> {
        loop {
            match self.peek() {
                Some(b'{') => {
                    self.bump();
                    return Some(Token::CurlyBraceOpen);
                },
                Some(b'}') => {
                    self.bump();
                    return Some(Token::CurlyBraceClose);
                },
                Some(b'[') => {
                    self.bump();
                    return Some(Token::BracketOpen);
                },
                Some(b']') => {
                    self.bump();
                    return Some(Token::BracketClose);
                },
                Some(b',') => {
                    self.bump();
                    return Some(Token::Comma);
                },
                Some(b':') => {
                    self.bump();
                    return Some(Token::Colon);
                },
                Some(b'"') => {
                    // Parse a string
                    let mut string_bytes = vec![];
                    self.bump();
                    while let Some(c) = self.peek() {
                        self.bump();
                        match c {
                            b'"' => {
                                // SAFETY: The internal byte buffer came from &str which is guaranteed to be utf-8
                                let string = unsafe { String::from_utf8_unchecked(string_bytes) };
                                return Some(Token::String(string));
                            },
                            b'\\' => {
                                match self.next()? {
                                    escaped_char @ (b'\\' | b'"' | b'/') => {
                                        string_bytes.push(escaped_char)
                                    },
                                    b'b' => string_bytes.push(b'\x08'),
                                    b'f' => string_bytes.push(b'\x0c'),
                                    b'n' => string_bytes.push(b'\n'),
                                    b'r' => string_bytes.push(b'\r'),
                                    b't' => string_bytes.push(b'\t'),
                                    b'u' => {
                                        let mut reference_value = self.consume_hex_escape()?;
                                        if (0xD800..=0xDBFF).contains(&reference_value) {
                                            // UTF-16 surrogate
                                            if self.next() != Some(b'\\') {
                                                return None;
                                            }
                                            if self.next() != Some(b'u') {
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
                                        string_bytes.extend_from_slice(
                                            referenced_char.encode_utf8(&mut [0_u8; 4]).as_bytes(),
                                        );
                                    },
                                    _ => {
                                        // Invalid escape character
                                        return None;
                                    },
                                }
                            },
                            other => {
                                string_bytes.push(other);
                            },
                        }
                    }

                    // EOF in string
                    return None;
                },
                Some(b' ' | b'\t' | b'\r' | b'\n') => {
                    // whitespace is skipped
                    self.bump();
                },
                Some(c @ (b'0'..=b'9')) => {
                    // Parse a numeric value
                    let mut num = (c - b'0') as u32;
                    self.bump();
                    while let Some(c) = self.peek() {
                        if c.is_ascii_digit() {
                            num *= 10;
                            num += (c - b'0') as u32;
                            self.bump();
                        } else {
                            break;
                        }
                    }

                    return Some(Token::Numeric(num));
                },
                Some(c @ (b't' | b'f' | b'n')) => {
                    // Parse an identifier
                    let mut ident_bytes = vec![c];
                    self.bump();
                    while let Some(c) = self.peek() {
                        if c.is_ascii_alphabetic() {
                            ident_bytes.push(c);
                            self.bump();
                        } else {
                            return match ident_bytes.as_slice() {
                                b"true" => Some(Token::True),
                                b"false" => Some(Token::False),
                                b"null" => Some(Token::Null),
                                _ => None,
                            };
                        }
                    }
                },
                Some(_) | None => return None,
            }
        }
    }

    fn peek_token(&mut self) -> Option<Token> {
        let old_position = self.position;
        let token = self.next_token();
        self.position = old_position;
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
        let old_position = self.position;
        if self.next_token() != Some(Token::Comma) {
            self.position = old_position;
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
