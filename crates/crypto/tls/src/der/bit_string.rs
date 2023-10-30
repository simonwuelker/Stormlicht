use std::fmt;

/// A arbitrary sequence of bits.
///
/// For the encoding inside DER, see <https://learn.microsoft.com/en-us/windows/win32/seccertenroll/about-bit-string>
#[derive(Clone)]
pub struct BitString {
    num_trailing_bits_to_skip: u8,
    bits: Vec<u8>,
}

#[derive(Clone, Copy, Debug)]
pub enum BitStringParseError {
    MissingLeadingByte,
    InvalidNumberofBitsToSkip,
}

impl TryFrom<&[u8]> for BitString {
    type Error = BitStringParseError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(BitStringParseError::MissingLeadingByte);
        }

        let num_trailing_bits_to_skip = value[0];

        if num_trailing_bits_to_skip > 7 {
            return Err(BitStringParseError::InvalidNumberofBitsToSkip);
        }

        let bit_string = Self {
            num_trailing_bits_to_skip,
            bits: value[1..].to_vec(),
        };

        Ok(bit_string)
    }
}

impl From<BitStringParseError> for super::Error {
    fn from(value: BitStringParseError) -> Self {
        Self::BitString(value)
    }
}

impl fmt::Debug for BitString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for full_byte in self.bits.iter().take(self.bits.len() - 1) {
            write!(f, "{full_byte:0>8b}")?;
        }

        if let Some(last) = self.bits.last() {
            // Write the first n bits of the last byte
            for n in (self.num_trailing_bits_to_skip..8).rev() {
                let bit = last >> n & 1;
                write!(f, "{bit}")?;
            }
        }

        Ok(())
    }
}
