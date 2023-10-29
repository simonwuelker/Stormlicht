mod integer;
mod item;
pub mod object_identifier;
mod sequence;

pub use integer::Integer;
pub use item::Item;
pub use object_identifier::ObjectIdentifier;
pub use sequence::Sequence;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClassTag {
    /// The type is native to ASN.1
    Universal,

    /// The type is only valid for one specific application
    Application,

    /// Meaning of this type depends on the context (such as within a sequence, set or choice)
    ContextSpecific,

    /// Defined in private specifications
    Private,
}

#[derive(Clone, Copy, Debug)]
pub enum PrimitiveOrConstructed {
    Primitive,
    Constructed,
}

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
    IllegalValue,
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
