use super::Error;

/// Implemented by data types that can be deserialized from DER data
pub trait Deserialize<'a>: Sized {
    type Error;

    fn deserialize(deserializer: &mut Deserializer<'a>) -> Result<Self, Self::Error>;

    fn from_bytes(bytes: &'a [u8], error: Self::Error) -> Result<Self, Self::Error> {
        let mut deserializer = Deserializer::new(bytes);
        let value = deserializer.parse_complete(error)?;

        Ok(value)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Deserializer<'a> {
    pub bytes: &'a [u8],
    pub ptr: usize,
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

/// Namespace for known universal type tags
///
/// Note that this can't be modeled as an enum, since
/// type tags can have any value (even one's that are
/// not defined anywhere), due to explicit/implicit tagging
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TypeTag(u32);

impl TypeTag {
    pub const END_OF_CONTENT: Self = Self(0);
    pub const BOOLEAN: Self = Self(1);
    pub const INTEGER: Self = Self(2);
    pub const BIT_STRING: Self = Self(3);
    pub const OCTET_STRING: Self = Self(4);
    pub const NULL: Self = Self(5);
    pub const OBJECT_IDENTIFIER: Self = Self(6);
    pub const OBJECT_DESCRIPTOR: Self = Self(7);
    pub const EXTERNAL: Self = Self(8);
    pub const REAL: Self = Self(9);
    pub const ENUMERATED: Self = Self(10);
    pub const EMBEDDED_PDV: Self = Self(11);
    pub const UTF8_STRING: Self = Self(12);
    pub const RELATIVE_OID: Self = Self(13);
    pub const TIME: Self = Self(14);
    pub const RESERVED: Self = Self(15);
    pub const SEQUENCE: Self = Self(16);
    pub const SET: Self = Self(17);
    pub const NUMERIC_STRING: Self = Self(18);
    pub const PRINTABLE_STRING: Self = Self(19);
    pub const T61_STRING: Self = Self(20);
    pub const VIDEOTEX_STRING: Self = Self(21);
    pub const IA5_STRING: Self = Self(22);
    pub const UTC_TIME: Self = Self(23);
    pub const GENERALIZED_TIME: Self = Self(24);
    pub const GRAPHIC_STRING: Self = Self(25);
    pub const VISIBLE_STRING: Self = Self(26);
    pub const GENERAL_STRING: Self = Self(27);
    pub const UNIVERSAL_STRING: Self = Self(28);
    pub const CHARACTER_STRING: Self = Self(29);
    pub const BMP_STRING: Self = Self(30);
    pub const DATE: Self = Self(31);
    pub const TIME_OF_DAY: Self = Self(32);
    pub const DATETIME: Self = Self(33);
    pub const DURATION: Self = Self(34);
    pub const OID_IRI: Self = Self(35);
    pub const RELATIVE_OID_IRI: Self = Self(36);
    pub const CONTEXT_SPECIFIC: Self = Self(37);

    pub fn new(n: u32) -> Self {
        Self(n)
    }
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

    pub fn is_exhausted(&self) -> bool {
        self.ptr == self.bytes.len()
    }

    pub fn expect_exhausted<T>(&self, error: T) -> Result<(), T> {
        if self.is_exhausted() {
            Ok(())
        } else {
            Err(error)
        }
    }

    pub fn parse<T: Deserialize<'a>>(&mut self) -> Result<T, T::Error> {
        T::deserialize(self)
    }

    pub fn parse_complete<T: Deserialize<'a>>(&mut self, error: T::Error) -> Result<T, T::Error> {
        let value: T = self.parse()?;
        if !self.is_exhausted() {
            return Err(error);
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

    pub fn peek_item_tag(&mut self) -> Result<TypeTag, Error> {
        let old_position = self.ptr;
        let byte = self.next_byte()?;

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

        self.ptr = old_position;

        let type_tag = TypeTag::new(type_tag);
        Ok(type_tag)
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
            type_tag: TypeTag::new(type_tag),
            bytes: value_bytes,
        };

        Ok(item)
    }
}
