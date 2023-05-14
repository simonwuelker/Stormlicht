use crate::ttf::TTFParseError;

#[derive(Clone, Copy, Debug)]
pub struct Stream<'a> {
    bytes: &'a [u8],
    ptr: usize,
}

impl<'a> Stream<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes: bytes,
            ptr: 0,
        }
    }

    pub fn read<T: Readable>(&mut self) -> Result<T, TTFParseError> {
        let value = T::read(&self.bytes[self.ptr..])?;
        self.ptr += T::SIZE;
        Ok(value)
    }

    pub fn skip_bytes(&mut self, num_bytes: usize) {
        self.ptr += num_bytes;
    }
}

/// Trait for things that can be read from a byte stream
pub trait Readable: Sized {
    const SIZE: usize = std::mem::size_of::<Self>();

    fn read(bytes: &[u8]) -> Result<Self, TTFParseError>;
}

impl Readable for u8 {
    fn read(bytes: &[u8]) -> Result<Self, TTFParseError> {
        if bytes.is_empty() {
            return Err(TTFParseError::UnexpectedEOF);
        }

        Ok(bytes[0])
    }
}

impl Readable for u16 {
    fn read(bytes: &[u8]) -> Result<Self, TTFParseError> {
        if bytes.len() < 2 {
            return Err(TTFParseError::UnexpectedEOF);
        }

        Ok(u16::from_be_bytes(bytes[..2].try_into().unwrap()))
    }
}

impl Readable for u32 {
    fn read(bytes: &[u8]) -> Result<Self, TTFParseError> {
        if bytes.len() < 4 {
            return Err(TTFParseError::UnexpectedEOF);
        }

        Ok(u32::from_be_bytes(bytes[..4].try_into().unwrap()))
    }
}

impl Readable for i8 {
    fn read(bytes: &[u8]) -> Result<Self, TTFParseError> {
        if bytes.is_empty() {
            return Err(TTFParseError::UnexpectedEOF);
        }

        Ok(i8::from_be_bytes([bytes[0]]))
    }
}

impl Readable for i16 {
    fn read(bytes: &[u8]) -> Result<Self, TTFParseError> {
        if bytes.len() < 2 {
            return Err(TTFParseError::UnexpectedEOF);
        }

        Ok(i16::from_be_bytes(bytes[..2].try_into().unwrap()))
    }
}

impl Readable for i32 {
    fn read(bytes: &[u8]) -> Result<Self, TTFParseError> {
        if bytes.len() < 4 {
            return Err(TTFParseError::UnexpectedEOF);
        }

        Ok(i32::from_be_bytes(bytes[..4].try_into().unwrap()))
    }
}
