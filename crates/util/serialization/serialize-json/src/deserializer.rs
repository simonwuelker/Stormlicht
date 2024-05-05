use sl_std::chars::ReversibleCharIterator;

use serialize::{
    deserialization::{EnumAccess, EnumVariantAccess, Error, MapAccess, SequentialAccess},
    Deserialize, Deserializer, Visitor,
};

#[derive(Clone, Debug)]
pub enum JsonError {
    Expected(&'static str),
    UnknownField(String),
    UnknownVariant(String),
    MissingField(&'static str),
    UnexpectedToken,
}

impl Error for JsonError {
    fn expected(expectation: &'static str) -> Self {
        Self::Expected(expectation)
    }

    fn unknown_field(field: String) -> Self {
        Self::UnknownField(field)
    }

    fn unknown_variant(name: String) -> Self {
        Self::UnknownVariant(name)
    }

    fn missing_field(field: &'static str) -> Self {
        Self::MissingField(field)
    }
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

#[derive(Clone, Copy)]
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

impl<'a> Deserializer for &mut JsonDeserializer<'a> {
    type Error = JsonError;

    fn deserialize_sequence<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor,
    {
        self.expect_next_token(Token::BracketOpen)?;

        let sequence = JsonSequence {
            done: false,
            deserializer: self,
        };

        visitor.visit_sequence(sequence)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor,
    {
        self.expect_next_token(Token::CurlyBraceOpen)?;

        let map = JsonMap {
            done: false,
            deserializer: self,
        };

        visitor.visit_map(map)
    }

    fn deserialize_struct<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor,
    {
        if let Some(Token::String(s)) = self.next_token() {
            visitor.visit_string(s)
        } else {
            Err(JsonError::UnexpectedToken)
        }
    }

    fn deserialize_usize<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor,
    {
        let Some(Token::Numeric(num)) = self.next_token() else {
            return Err(JsonError::UnexpectedToken);
        };

        visitor.visit_usize(num as usize)
    }

    fn deserialize_enum<V: Visitor>(self, visitor: V) -> Result<V::Value, Self::Error> {
        // Enums are stored as { variant: data}
        self.expect_next_token(Token::CurlyBraceOpen)?;

        let enumeration = JsonEnum { deserializer: self };

        visitor.visit_enum(enumeration)
    }

    fn deserialize_option<V: Visitor>(self, visitor: V) -> Result<V::Value, Self::Error> {
        if self.peek_token() == Some(Token::Null) {
            _ = self.next_token();

            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_bool<V: Visitor>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.next_token() {
            Some(Token::True) => visitor.visit_bool(true),
            Some(Token::False) => visitor.visit_bool(false),
            _ => return Err(JsonError::UnexpectedToken),
        }
    }
}

struct JsonSequence<'a, 'b> {
    done: bool,
    deserializer: &'a mut JsonDeserializer<'b>,
}

impl<'a, 'b> SequentialAccess for JsonSequence<'a, 'b> {
    type Error = <&'b mut JsonDeserializer<'b> as Deserializer>::Error;

    fn next_element<T>(&mut self) -> Result<Option<T>, Self::Error>
    where
        T: Deserialize,
    {
        if self.deserializer.peek_token() == Some(Token::BracketClose) {
            let _ = self.deserializer.next_token();
            self.done = true;
        }

        if self.done {
            return Ok(None);
        }

        let value = T::deserialize(&mut *self.deserializer)?;

        if self.deserializer.peek_token() == Some(Token::Comma) {
            _ = self.deserializer.next_token();

            // The sequence *may* end with a trailing comma
            if self.deserializer.peek_token() == Some(Token::BracketClose) {
                _ = self.deserializer.next_token();
                self.done = true;
            }
        } else {
            // If there is no comma then the sequence is finished
            self.deserializer.expect_next_token(Token::BracketClose)?;
            self.done = true;
        }

        Ok(Some(value))
    }
}

struct JsonMap<'a, 'b> {
    done: bool,
    deserializer: &'a mut JsonDeserializer<'b>,
}

impl<'a, 'b> MapAccess for JsonMap<'a, 'b> {
    type Error = <&'b mut JsonDeserializer<'b> as Deserializer>::Error;

    fn next_key<K>(&mut self) -> Result<Option<K>, Self::Error>
    where
        K: Deserialize,
    {
        if self.deserializer.peek_token() == Some(Token::CurlyBraceClose) {
            let _ = self.deserializer.next_token();
            self.done = true;
        }

        if self.done {
            return Ok(None);
        }

        let key = K::deserialize(&mut *self.deserializer)?;

        Ok(Some(key))
    }

    fn next_value<V>(&mut self) -> Result<V, Self::Error>
    where
        V: Deserialize,
    {
        self.deserializer.expect_next_token(Token::Colon)?;
        let value = V::deserialize(&mut *self.deserializer)?;

        if self.deserializer.peek_token() == Some(Token::Comma) {
            _ = self.deserializer.next_token();

            // The map *may* end with a trailing comma
            if self.deserializer.peek_token() == Some(Token::CurlyBraceClose) {
                _ = self.deserializer.next_token();
                self.done = true;
            }
        } else {
            // If there is no comma then the map is finished
            self.deserializer
                .expect_next_token(Token::CurlyBraceClose)?;
            self.done = true;
        }

        Ok(value)
    }
}

struct JsonEnum<'a, 'b> {
    deserializer: &'a mut JsonDeserializer<'b>,
}

struct JsonEnumVariant<'a, 'b> {
    deserializer: &'a mut JsonDeserializer<'b>,
}

impl<'a, 'b> EnumAccess for JsonEnum<'a, 'b> {
    type Error = <&'b mut JsonDeserializer<'b> as Deserializer>::Error;
    type Variant = JsonEnumVariant<'a, 'b>;

    fn variant<V>(self) -> Result<(V, Self::Variant), Self::Error>
    where
        V: Deserialize,
    {
        let variant = V::deserialize(&mut *self.deserializer)?;
        self.deserializer.expect_next_token(Token::Colon)?;

        // What follows is either a sequence (tuple enum variant) or a map (struct enum variant)
        let variant_data = JsonEnumVariant {
            deserializer: &mut *self.deserializer,
        };

        Ok((variant, variant_data))
    }
}

impl<'a, 'b> EnumVariantAccess for JsonEnumVariant<'a, 'b> {
    type Error = <&'b mut JsonDeserializer<'b> as Deserializer>::Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        self.deserializer.expect_next_token(Token::BracketOpen)?;
        self.deserializer.expect_next_token(Token::BracketClose)?;
        self.deserializer
            .expect_next_token(Token::CurlyBraceClose)?;

        Ok(())
    }

    fn tuple_variant<V: Visitor>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let value = self.deserializer.deserialize_sequence(visitor)?;

        self.deserializer
            .expect_next_token(Token::CurlyBraceClose)?;

        Ok(value)
    }

    fn struct_variant<V: Visitor>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let value = self.deserializer.deserialize_struct(visitor)?;

        self.deserializer
            .expect_next_token(Token::CurlyBraceClose)?;

        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
