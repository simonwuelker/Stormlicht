use super::{Deserializer, Error, Primitive, TypeTag};

#[derive(Clone, Copy, Debug)]
pub struct Sequence<'a> {
    bytes: &'a [u8],
}

impl<'a> Sequence<'a> {
    pub fn deserializer(&self) -> Deserializer<'a> {
        Deserializer::new(self.bytes)
    }
}

impl<'a> Primitive<'a> for Sequence<'a> {
    type Error = Error;

    const TYPE_TAG: TypeTag = TypeTag::SEQUENCE;

    fn from_value_bytes(bytes: &'a [u8]) -> Result<Self, Self::Error> {
        Ok(Self { bytes })
    }
}
