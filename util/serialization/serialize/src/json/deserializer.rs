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

    fn bump(&mut self) {
        self.position += 1;
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
                        if c == b'"' {
                            // SAFETY: The internal byte buffer came from &str which is guaranteed to be utf-8
                            let string = unsafe { String::from_utf8_unchecked(string_bytes) };
                            return Some(Token::String(string));
                        } else {
                            string_bytes.push(c);
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
}
