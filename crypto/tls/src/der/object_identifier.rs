use std::fmt;

#[derive(Clone)]
pub struct ObjectIdentifier {
    parts: Vec<usize>,
}

impl TryFrom<&[u8]> for ObjectIdentifier {
    type Error = super::Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.is_empty() {
            return Err(Self::Error::UnexpectedEOF);
        }

        let v1 = bytes[0] / 40;
        let v2 = bytes[0] % 40;

        let is_valid = matches!((v1, v2), (0 | 1, ..=39) | (2, _));
        if !is_valid {
            return Err(Self::Error::IllegalValue);
        }

        let mut parts = vec![v1 as usize, v2 as usize];
        let mut remaining_bytes = bytes[1..].iter();
        while let Some(byte) = remaining_bytes.next() {
            let mut has_more = byte >> 7 != 0;
            let mut value = (byte & 0b01111111) as usize;

            while has_more {
                let byte = remaining_bytes.next().ok_or(super::Error::UnexpectedEOF)?;
                value <<= 7;
                value |= (byte & 0b01111111) as usize;
                has_more = byte >> 7 != 0;
            }

            parts.push(value);
        }

        Ok(Self { parts })
    }
}

impl fmt::Debug for ObjectIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = self
            .parts
            .iter()
            .map(usize::to_string)
            .collect::<Vec<String>>()
            .join(".");
        write!(f, "{s}")
    }
}
