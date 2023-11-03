mod bit_string;
mod integer;
mod item;
pub mod object_identifier;
mod reader;
mod sequence;

pub use bit_string::{BitString, BitStringParseError};
pub use integer::Integer;
pub use item::Item;
pub use object_identifier::ObjectIdentifier;
pub use reader::{ClassTag, Deserialize, Deserializer, PrimitiveOrConstructed, TypeTag};
pub use sequence::Sequence;

#[derive(Clone, Copy, Debug)]
pub enum Error {
    ReservedTypeTag,
    UnknownTypeTag,
    /// A indefinite length was encountered
    ///
    /// This is allowed in BER but not DER
    IndefiniteLength,
    ReservedLength,
    UnexpectedEOF,

    /// An error occured while trying to parse a [BitString]
    BitString(BitStringParseError),
    IllegalValue,
    UnknownObjectIdentifer,
    TrailingBytes,
}

pub trait Parse: Sized {
    type Error: From<Error>;

    fn try_from_item(item: Item<'_>) -> Result<Self, Self::Error>;

    fn try_parse(bytes: &[u8]) -> Result<(Self, &[u8]), Self::Error> {
        let (item, length) = Item::parse(bytes)?;
        let parsed_value = Self::try_from_item(item)?;
        Ok((parsed_value, &bytes[length..]))
    }
}

impl From<object_identifier::UnknownObjectIdentifier> for Error {
    fn from(_: object_identifier::UnknownObjectIdentifier) -> Self {
        Self::UnknownObjectIdentifer
    }
}
