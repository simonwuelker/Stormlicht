use super::Error;

/// Implemented by data types that can be deserialized from DER data
pub trait Deserialize: Sized {
    type Error;

    fn deserialize(deserializer: &mut Deserializer<'_>) -> Result<Self, Self::Error>;
}

#[derive(Clone, Copy, Debug)]
pub struct Deserializer<'a> {
    bytes: &'a [u8],
    ptr: usize,
}

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TypeTag {
    EndOfContent,
    Boolean,
    Integer,
    BitString,
    OctetString,
    Null,
    ObjectIdentifier,
    ObjectDescriptor,
    External,
    Real,
    Enumerated,
    EmbeddedPDV,
    Utf8String,
    RelativeOID,
    Time,
    Sequence,
    Set,
    NumericString,
    PrintableString,
    T61String,
    VideotexString,
    IA5String,
    UtcTime,
    GeneralizedTime,
    GraphicString,
    VisibleString,
    GeneralString,
    UniversalString,
    CharacterString,
    BMPString,
    Date,
    TimeOfDay,
    DateTime,
    Duration,
    OidIri,
    RelativeOidIri,
    ContextSpecific,
}

#[derive(Clone, Copy, Debug)]
struct Item<'a> {
    type_tag: TypeTag,
    bytes: &'a [u8],
}

#[derive(Clone, Copy, Debug)]
pub enum PrimitiveOrConstructed {
    Primitive,
    Constructed,
}

impl<'a> Deserializer<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, ptr: 0 }
    }

    pub fn is_done(&self) -> bool {
        self.ptr == self.bytes.len()
    }

    pub fn expect_next<T: Deserialize>(&mut self) -> Result<T, <T as Deserialize>::Error> {
        T::deserialize(self)
    }

    pub fn parse_complete<T: Deserialize>(&mut self) -> Result<T, Error>
    where
        Error: From<T::Error>,
    {
        let value = self.expect_next()?;
        if self.is_done() {
            return Err(Error::TrailingBytes);
        }
        Ok(value)
    }

    #[inline]
    fn next_byte(&mut self) -> Result<u8, Error> {
        let byte = *self.bytes.get(self.ptr).ok_or(Error::UnexpectedEOF)?;
        self.ptr += 1;
        Ok(byte)
    }

    fn advance_by(&mut self, n: usize) {
        self.ptr += n
    }

    pub fn expect_next_item_and_get_value(
        &mut self,
        expected_type: TypeTag,
    ) -> Result<&'a [u8], Error> {
        let item = self.next_primitive_item()?;

        if item.type_tag != expected_type {
            log::warn!("Expected {:?} but found {:?}", expected_type, item.type_tag);
            return Err(Error::IllegalValue);
        }

        Ok(item.bytes)
    }

    fn next_primitive_item(&mut self) -> Result<Item<'a>, Error> {
        let byte = self.next_byte()?;

        let _class_tag = match byte >> 6 {
            0 => ClassTag::Universal,
            1 => ClassTag::Application,
            2 => ClassTag::ContextSpecific,
            3 => ClassTag::Private,
            _ => unreachable!(),
        };

        let _pc = if byte & 0b00100000 == 0 {
            PrimitiveOrConstructed::Primitive
        } else {
            PrimitiveOrConstructed::Constructed
        };

        let type_tag = if byte & 0b00011111 == 31 {
            // Type tag is encoded using long form
            let mut done = false;
            let mut type_tag_value = 0;
            while !done {
                let byte = self.next_byte()?;

                type_tag_value <<= 7;
                type_tag_value |= (byte & 0b01111111) as u32;

                done = byte & 0b1000000 == 0;
            }
            type_tag_value
        } else {
            (byte & 0b00011111) as u32
        };

        // Read item length
        let byte = self.next_byte()?;

        let first_bit = byte & 0b10000000 != 0;
        let remaining = byte & 0b01111111;
        let length = match (first_bit, remaining) {
            (false, _) => {
                // Definite short form
                remaining as usize
            },
            (true, 0) => return Err(Error::IndefiniteLength),
            (true, 127) => return Err(Error::ReservedLength),
            (true, number_of_octets) => {
                // Definite long form
                // First byte defines the number of length bytes that follow
                const N_BYTES: u8 = (usize::BITS / 8) as u8;
                let mut buffer = [0; N_BYTES as usize];

                // Never read more than 8 bytes.
                // If your type is longer than u64::MAX, then you likely
                // have a problem anyways
                let number_of_octets = number_of_octets.min(N_BYTES);
                let n_bytes_to_skip = N_BYTES - number_of_octets;

                let int_bytes = &self
                    .bytes
                    .get(self.ptr..self.ptr + number_of_octets as usize)
                    .ok_or(Error::UnexpectedEOF)?;
                buffer[n_bytes_to_skip as usize..].copy_from_slice(int_bytes);

                self.advance_by(number_of_octets as usize);
                usize::from_be_bytes(buffer)
            },
        };

        let value_bytes = self
            .bytes
            .get(self.ptr..self.ptr + length)
            .ok_or(Error::UnexpectedEOF)?;
        self.advance_by(length);

        let item = Item {
            type_tag: TypeTag::try_from(type_tag)?,
            bytes: value_bytes,
        };

        Ok(item)
    }
}

impl TryFrom<u32> for TypeTag {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        let type_tag = match value {
            0 => TypeTag::EndOfContent,
            1 => TypeTag::Boolean,
            2 => TypeTag::Integer,
            3 => TypeTag::BitString,
            4 => TypeTag::OctetString,
            5 => TypeTag::Null,
            6 => TypeTag::ObjectIdentifier,
            7 => TypeTag::ObjectDescriptor,
            8 => TypeTag::External,
            9 => TypeTag::Real,
            10 => TypeTag::Enumerated,
            11 => TypeTag::EmbeddedPDV,
            12 => TypeTag::Utf8String,
            13 => TypeTag::RelativeOID,
            14 => TypeTag::Time,
            15 => return Err(Error::ReservedTypeTag),
            16 => TypeTag::Sequence,
            17 => TypeTag::Set,
            18 => TypeTag::NumericString,
            19 => TypeTag::PrintableString,
            20 => TypeTag::T61String,
            21 => TypeTag::VideotexString,
            22 => TypeTag::IA5String,
            23 => TypeTag::UtcTime,
            24 => TypeTag::GeneralizedTime,
            25 => TypeTag::GraphicString,
            26 => TypeTag::VisibleString,
            27 => TypeTag::GeneralString,
            28 => TypeTag::UniversalString,
            29 => TypeTag::CharacterString,
            30 => TypeTag::BMPString,
            31 => TypeTag::Date,
            32 => TypeTag::TimeOfDay,
            33 => TypeTag::DateTime,
            34 => TypeTag::Duration,
            35 => TypeTag::OidIri,
            36 => TypeTag::RelativeOidIri,
            _ => return Err(Error::UnknownTypeTag),
        };

        Ok(type_tag)
    }
}
