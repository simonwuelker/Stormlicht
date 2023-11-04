use super::{Deserialize, Deserializer, Error, TypeTag};

#[derive(Clone, Copy, Debug)]
pub struct Sequence<'a> {
    bytes: &'a [u8],
}

impl<'a> Sequence<'a> {
    pub fn deserializer(&self) -> Deserializer<'a> {
        Deserializer::new(self.bytes)
    }
}

impl<'a> Deserialize<'a> for Sequence<'a> {
    type Error = Error;

    fn deserialize(deserializer: &mut Deserializer<'a>) -> Result<Self, Self::Error> {
        let bytes = deserializer.expect_next_item_and_get_value(TypeTag::SEQUENCE)?;
        let sequence = Self { bytes };
        Ok(sequence)
    }
}
