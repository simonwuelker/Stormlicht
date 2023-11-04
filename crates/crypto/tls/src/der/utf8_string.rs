use super::{Deserialize, Deserializer, Error, TypeTag};

#[derive(Clone, Debug)]
pub struct Utf8String {
    contents: String,
}

impl From<Utf8String> for String {
    fn from(value: Utf8String) -> Self {
        value.contents
    }
}

impl<'a> Deserialize<'a> for Utf8String {
    type Error = Error;

    fn deserialize(deserializer: &mut Deserializer<'a>) -> Result<Self, Self::Error> {
        let bytes = deserializer.expect_next_item_and_get_value(TypeTag::UTF8_STRING)?;

        let contents = String::from_utf8(bytes.to_vec()).map_err(|_| Error::IllegalValue)?;
        let utf8_string = Self { contents };

        Ok(utf8_string)
    }
}
