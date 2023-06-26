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
    source: &'a str,
    position: usize,
}

impl<'a> JsonDeserializer<'a> {
    pub fn new(json: &'a str) -> Self {
        Self {
            source: json,
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

    fn peek_char(&self) -> Option<char> {
        self.source.chars().nth(self.position)
    }

    fn bump(&mut self) {
        self.position += 1;
    }

    fn next_token(&mut self) -> Option<Token> {
        loop {
            match self.peek_char() {
                Some('{') => {
                    self.bump();
                    return Some(Token::CurlyBraceOpen);
                },
                Some('}') => {
                    self.bump();
                    return Some(Token::CurlyBraceClose);
                },
                Some('[') => {
                    self.bump();
                    return Some(Token::BracketOpen);
                },
                Some(']') => {
                    self.bump();
                    return Some(Token::BracketClose);
                },
                Some(',') => {
                    self.bump();
                    return Some(Token::Comma);
                },
                Some(':') => {
                    self.bump();
                    return Some(Token::Colon);
                },
                Some('"') => {
                    // Parse a string
                    let mut s = String::new();
                    self.bump();
                    while let Some(c) = self.peek_char() {
                        self.bump();
                        if c == '"' {
                            return Some(Token::String(s));
                        } else {
                            s.push(c);
                        }
                    }

                    // EOF in string
                    return None;
                },
                Some(' ' | '\t' | '\r' | '\n') => {
                    // whitespace is skipped
                    self.bump();
                },
                Some(c @ ('0'..='9')) => {
                    // Parse a numeric value
                    let mut num = c.to_digit(10).expect("Digits 0-9 are valid base 10 digits");
                    self.bump();
                    while let Some(c) = self.peek_char() {
                        if c.is_ascii_digit() {
                            num *= 10;
                            num += c.to_digit(10).expect("Digits 0-9 are valid base 10 digits");
                            self.bump();
                        } else {
                            break;
                        }
                    }

                    return Some(Token::Numeric(num));
                },
                Some(c) => {
                    // Parse an identifier
                    let mut s = c.to_string();
                    self.bump();
                    while let Some(c) = self.peek_char() {
                        if c.is_ascii_alphabetic() {
                            s.push(c);
                            self.bump();
                        } else {
                            return match s.as_ref() {
                                "true" => Some(Token::True),
                                "false" => Some(Token::False),
                                "null" => Some(Token::Null),
                                _ => None,
                            };
                        }
                    }
                },
                None => return None,
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
