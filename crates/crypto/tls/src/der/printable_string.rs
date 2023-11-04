use super::{Error, Primitive, TypeTag};
use sl_std::ascii;

#[derive(Clone, Debug)]
pub struct PrintableString {
    contents: ascii::String,
}

impl<'a> Primitive<'a> for PrintableString {
    type Error = Error;

    const TYPE_TAG: TypeTag = TypeTag::PRINTABLE_STRING;

    fn from_value_bytes(bytes: &'a [u8]) -> Result<Self, Self::Error> {
        let contents = ascii::String::from_bytes(bytes.to_vec()).ok_or(Error::IllegalValue)?;
        if !contents
            .chars()
            .iter()
            .copied()
            .all(is_printablestring_char)
        {
            return Err(Error::IllegalValue);
        }

        let printable_string = Self { contents };
        Ok(printable_string)
    }
}

impl From<PrintableString> for ascii::String {
    fn from(value: PrintableString) -> Self {
        value.contents
    }
}

impl From<PrintableString> for String {
    fn from(value: PrintableString) -> Self {
        value.contents.into()
    }
}

/// Whether or not a character can occur inside [Item::PrintableString]
#[inline]
#[must_use]
fn is_printablestring_char(c: ascii::Char) -> bool {
    matches!(c.to_char(), 'A'..='Z' | 'a'..='z' | '0'..='9' | ' ' | '\'' | '(' | ')' | '+' | ',' | '-' | '.' | '/' | ':' | '=' | '?')
}
