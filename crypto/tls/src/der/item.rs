use super::{ClassTag, Error, Integer, ObjectIdentifier, PrimitiveOrConstructed, Sequence};

#[derive(Clone, Debug)]
pub enum Item<'a> {
    EndOfContent,
    Boolean,
    Integer(Integer),
    BitString,
    OctetString,
    Null,
    ObjectIdentifier(ObjectIdentifier),
    ObjectDescriptor,
    External,
    Real,
    Enumerated,
    EmbeddedPDV,
    Utf8String(String),
    RelativeOID,
    Time,
    Sequence(Sequence<'a>),
    Set(Sequence<'a>),
    NumericString,
    PrintableString(String),
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
                2 => Item::Integer(Integer::from_be_bytes(value_bytes)),
                3 => Item::BitString,
                4 => Item::OctetString,
                5 => Item::Null,
                6 => Item::ObjectIdentifier(ObjectIdentifier::try_from(value_bytes)?),
                7 => Item::ObjectDescriptor,
                8 => Item::External,
                9 => Item::Real,
                10 => Item::Enumerated,
                11 => Item::EmbeddedPDV,
                12 => {
                    let string =
                        String::from_utf8(value_bytes.to_vec()).map_err(|_| Error::IllegalValue)?;
                    Item::Utf8String(string)
                },
                13 => Item::RelativeOID,
                14 => Item::Time,
                15 => return Err(Error::ReservedTypeTag),
                16 => Item::Sequence(Sequence::new(value_bytes)),
                17 => Item::Set(Sequence::new(value_bytes)),
                18 => Item::NumericString,
                19 => {
                    let string =
                        String::from_utf8(value_bytes.to_vec()).map_err(|_| Error::IllegalValue)?;
                    if !string.chars().all(is_printablestring_char) {
                        return Err(Error::IllegalValue);
                    }
                    Item::PrintableString(string)
                },
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

/// Whether or not a character can occur inside [Item::PrintableString]
#[inline]
#[must_use]
fn is_printablestring_char(c: char) -> bool {
    matches!(c, 'A'..='Z' | 'a'..='z' | '0'..='9' | ' ' | '\'' | '(' | ')' | '+' | ',' | '-' | '.' | '/' | ':' | '=' | '?')
}
