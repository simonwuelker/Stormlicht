pub struct BitReader<'a> {
    bytes: &'a [u8],

    /// Offset in bits
    offset: usize,
}

impl<'a> BitReader<'a> {
    #[must_use]
    pub const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    #[inline]
    #[must_use]
    const fn byte_offset(&self) -> usize {
        self.offset >> 3
    }

    #[inline]
    #[must_use]
    const fn bit_offset(&self) -> u8 {
        (self.offset & 0b111) as u8
    }

    pub fn advance(&mut self, bits: usize) {
        self.offset += bits;
    }

    /// Look at the next 16 bits in the stream (extending with zeros if necessary)
    ///
    /// Doesn't advance the internal reader
    pub fn peek_u16(&self) -> u16 {
        // We will at most need three bytes
        let first_byte = self
            .bytes
            .get(self.byte_offset())
            .copied()
            .unwrap_or_default();
        let second_byte = self
            .bytes
            .get(self.byte_offset() + 1)
            .copied()
            .unwrap_or_default();
        let third_byte = self
            .bytes
            .get(self.byte_offset() + 1)
            .copied()
            .unwrap_or_default();

        let mut result = (second_byte as u16) << 8 | (third_byte as u16);

        let bits_from_first_byte = 8 - self.bit_offset();
        let first_mask = (1_u8 << bits_from_first_byte) - 1;

        // Put the bits from the first byte at the very front
        result >>= bits_from_first_byte;
        result |= ((first_byte & first_mask) as u16) << (16 - bits_from_first_byte);

        result
    }
}
