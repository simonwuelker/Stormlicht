use super::{Error, Primitive, TypeTag};

#[derive(Clone, Debug)]
pub struct Utf8String {
    contents: String,
}

impl From<Utf8String> for String {
    fn from(value: Utf8String) -> Self {
        value.contents
    }
}

impl<'a> Primitive<'a> for Utf8String {
    type Error = Error;

    const TYPE_TAG: TypeTag = TypeTag::UTF8_STRING;

    fn from_value_bytes(bytes: &'a [u8]) -> Result<Self, Self::Error> {
        let contents = String::from_utf8(bytes.to_vec()).map_err(|_| Error::IllegalValue)?;
        let utf8_string = Self { contents };

        Ok(utf8_string)
    }
}
