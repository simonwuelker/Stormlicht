mod bit_string;
mod integer;
mod object_identifier;
mod printable_string;
mod reader;
mod sequence;
mod set;
mod utc_time;
mod utf8_string;

pub use bit_string::{BitString, BitStringParseError};
pub use integer::Integer;
pub use object_identifier::ObjectIdentifier;
pub use printable_string::PrintableString;
pub use reader::{ClassTag, Deserialize, Deserializer, PrimitiveOrConstructed, TypeTag};
pub use sequence::Sequence;
pub use set::Set;
pub use utc_time::UtcTime;
pub use utf8_string::Utf8String;

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

impl From<object_identifier::UnknownObjectIdentifier> for Error {
    fn from(_: object_identifier::UnknownObjectIdentifier) -> Self {
        Self::UnknownObjectIdentifer
    }
}
