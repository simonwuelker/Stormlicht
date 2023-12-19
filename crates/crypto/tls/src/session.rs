use crate::encoding::{self, Cursor, Decoding, Encoding};

#[derive(Clone, Debug)]
pub struct SessionId {
    // Up to 32 bytes long
    bytes: Vec<u8>,
}

impl SessionId {
    #[must_use]
    pub fn empty() -> Self {
        Self { bytes: vec![] }
    }
}

impl Encoding for SessionId {
    fn encode(&self, bytes: &mut Vec<u8>) {
        (self.bytes.len() as u8).encode(bytes);
        bytes.extend_from_slice(&self.bytes);
    }
}

impl<'a> Decoding<'a> for SessionId {
    fn decode(cursor: &mut Cursor<'a>) -> encoding::Result<Self> {
        let id_length: u8 = cursor.decode()?;

        if 32 < id_length {
            return Err(encoding::Error);
        }

        let remainder = cursor.remainder();
        if remainder.len() < id_length as usize {
            return Err(encoding::Error);
        }

        let bytes = remainder[..id_length as usize].to_owned();
        cursor.advance(id_length as usize);

        Ok(Self { bytes })
    }
}
