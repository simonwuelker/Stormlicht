use std::fmt;

pub trait Serializer: Sized + fmt::Write {
    fn serialize<T: Serialize>(&mut self, value: T) -> fmt::Result {
        value.serialize_to(self)
    }

    /// <https://www.w3.org/TR/cssom-1/#serialize-an-identifier>
    fn serialize_identifier(&mut self, identifier: &str) -> fmt::Result {
        let mut first_was_dash = false;

        for (index, c) in identifier.chars().enumerate() {
            #[allow(clippy::if_same_then_else)] // Spec things...
            if c == '\x00' {
                self.serialize(char::REPLACEMENT_CHARACTER)?
            } else if ('\x01'..='\x1F').contains(&c) || c == '\x7F' {
                for escape_char in c.escape_unicode() {
                    self.serialize(escape_char)?
                }
            } else if index == 0 && c.is_ascii_digit() {
                for escape_char in c.escape_unicode() {
                    self.serialize(escape_char)?
                }
            } else if index == 1 && first_was_dash && c.is_ascii_digit() {
                for escape_char in c.escape_unicode() {
                    self.serialize(escape_char)?
                }
            } else if index == 0 && c == '-' && identifier.len() == 1 {
                self.serialize('\\')?;
                self.serialize(c)?;
            } else if matches!(c, '\u{0080}'..|'-'| '_') || c.is_ascii_alphanumeric() {
                self.serialize(c)?;
            } else {
                self.serialize('\\')?;
                self.serialize(c)?;
            }

            if index == 0 && c == '-' {
                first_was_dash = true;
            }
        }

        Ok(())
    }

    /// <https://www.w3.org/TR/cssom-1/#serialize-a-comma-separated-list>
    fn serialize_comma_seperated_list<T, I>(&mut self, list: I) -> fmt::Result
    where
        I: IntoIterator<Item = T>,
        T: Serialize,
    {
        self.serialize_list_with_separator(list, ", ")
    }

    /// <https://www.w3.org/TR/cssom-1/#serialize-a-whitespace-separated-list>
    fn serialize_whitespace_seperated_list<T, I>(&mut self, list: I) -> fmt::Result
    where
        I: IntoIterator<Item = T>,
        T: Serialize,
    {
        self.serialize_list_with_separator(list, ' ')
    }

    fn serialize_list_with_separator<T, I, S>(&mut self, list: I, separator: S) -> fmt::Result
    where
        I: IntoIterator<Item = T>,
        T: Serialize,
        S: Serialize + Copy,
    {
        let mut iterator = list.into_iter();

        if let Some(first_item) = iterator.next() {
            self.serialize(first_item)?;
        }

        for item in iterator {
            self.serialize(separator)?;
            self.serialize(item)?;
        }

        Ok(())
    }
}

impl Serializer for String {}
impl Serializer for &mut String {}

pub trait Serialize {
    fn serialize_to<T: Serializer>(&self, serializer: &mut T) -> fmt::Result;

    fn serialize_to_string(&self) -> Result<String, fmt::Error> {
        let mut result = String::new();
        self.serialize_to(&mut result)?;
        Ok(result)
    }
}

impl Serialize for &str {
    fn serialize_to<T: Serializer>(&self, serializer: &mut T) -> fmt::Result {
        serializer.write_str(self)
    }
}

impl Serialize for char {
    fn serialize_to<T: Serializer>(&self, serializer: &mut T) -> fmt::Result {
        serializer.write_char(*self)
    }
}
