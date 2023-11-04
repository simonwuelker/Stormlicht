use super::{Deserialize, Deserializer, Error, TypeTag};

#[derive(Clone, Copy, Debug)]
pub struct Set<'a> {
    bytes: &'a [u8],
}

impl<'a> Set<'a> {
    pub fn deserializer(&self) -> Deserializer<'a> {
        Deserializer::new(self.bytes)
    }
}

impl<'a> Deserialize<'a> for Set<'a> {
    type Error = Error;

    fn deserialize(deserializer: &mut Deserializer<'a>) -> Result<Self, Self::Error> {
        let bytes = deserializer.expect_next_item_and_get_value(TypeTag::Set)?;
        let set = Self { bytes };
        Ok(set)
    }
}
