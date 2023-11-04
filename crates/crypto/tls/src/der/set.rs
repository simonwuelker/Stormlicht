use super::{Deserializer, Error, Primitive, TypeTag};

#[derive(Clone, Copy, Debug)]
pub struct Set<'a> {
    bytes: &'a [u8],
}

impl<'a> Set<'a> {
    pub fn deserializer(&self) -> Deserializer<'a> {
        Deserializer::new(self.bytes)
    }
}

impl<'a> Primitive<'a> for Set<'a> {
    type Error = Error;

    const TYPE_TAG: TypeTag = TypeTag::SET;

    fn from_value_bytes(bytes: &'a [u8]) -> Result<Self, Self::Error> {
        Ok(Self { bytes })
    }
}
