use std::iter::FusedIterator;

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
pub enum Item<'a> {
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
    Sequence(Sequence<'a>),
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
    ContextSpecific(&'a [u8]),
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
}

impl<'a> Item<'a> {
    pub fn parse(bytes: &'a [u8]) -> Result<(Self, usize), Error> {
        // Read item identifier
        let mut index = 0;
        let byte = bytes.get(index).ok_or(Error::UnexpectedEOF)?;
        index += 1;

        let class_tag = match byte >> 6 {
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
                let byte = bytes.get(index).ok_or(Error::UnexpectedEOF)?;
                index += 1;

                type_tag_value <<= 7;
                type_tag_value |= (byte & 0b01111111) as u32;

                done = byte & 0b1000000 == 0;
            }
            type_tag_value
        } else {
            (byte & 0b00011111) as u32
        };

        // Read item length
        let byte = bytes.get(index).ok_or(Error::UnexpectedEOF)?;
        index += 1;

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

                let int_bytes = &bytes
                    .get(index..index + number_of_octets as usize)
                    .ok_or(Error::UnexpectedEOF)?;
                buffer[n_bytes_to_skip as usize..].copy_from_slice(int_bytes);

                index += number_of_octets as usize;
                usize::from_be_bytes(buffer)
            },
        };

        let value_bytes = bytes
            .get(index..index + length)
            .ok_or(Error::UnexpectedEOF)?;
        index += length;

        let item = if class_tag == ClassTag::ContextSpecific {
            Item::ContextSpecific(value_bytes)
        } else {
            // Construct the actual item
            match type_tag {
                0 => Item::EndOfContent,
                1 => Item::Boolean,
                2 => Item::Integer,
                3 => Item::BitString,
                4 => Item::OctetString,
                5 => Item::Null,
                6 => Item::ObjectIdentifier,
                7 => Item::ObjectDescriptor,
                8 => Item::External,
                9 => Item::Real,
                10 => Item::Enumerated,
                11 => Item::EmbeddedPDV,
                12 => Item::Utf8String,
                13 => Item::RelativeOID,
                14 => Item::Time,
                15 => return Err(Error::ReservedTypeTag),
                16 => Item::Sequence(Sequence { bytes: value_bytes }),
                17 => Item::Set,
                18 => Item::NumericString,
                19 => Item::PrintableString,
                20 => Item::T61String,
                21 => Item::VideotexString,
                22 => Item::IA5String,
                23 => Item::UtcTime,
                24 => Item::GeneralizedTime,
                25 => Item::GraphicString,
                26 => Item::VisibleString,
                27 => Item::GeneralString,
                28 => Item::UniversalString,
                29 => Item::CharacterString,
                30 => Item::BMPString,
                31 => Item::Date,
                32 => Item::TimeOfDay,
                33 => Item::DateTime,
                34 => Item::Duration,
                35 => Item::OidIri,
                36 => Item::RelativeOidIri,
                _ => return Err(Error::UnknownTypeTag),
            }
        };

        Ok((item, index))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Sequence<'a> {
    bytes: &'a [u8],
}

impl<'a> Iterator for Sequence<'a> {
    type Item = Result<Item<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bytes.is_empty() {
            return None;
        }

        match Item::parse(self.bytes) {
            Ok((item, length)) => {
                self.bytes = &self.bytes[length..];
                Some(Ok(item))
            },
            Err(e) => Some(Err(e)),
        }
    }
}

impl<'a> FusedIterator for Sequence<'a> {}
