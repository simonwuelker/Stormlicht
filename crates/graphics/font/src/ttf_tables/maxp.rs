//! [MaxP](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6maxp.html) table implementation.

use crate::ttf::read_u16_at;

#[derive(Clone, Copy, Debug)]
pub struct MaxPTable {
    num_glyphs: u16,
}

impl MaxPTable {
    #[inline]
    #[must_use]
    pub fn new(data: &[u8]) -> Self {
        Self {
            num_glyphs: read_u16_at(data, 4),
        }
    }

    /// Get the number of glyphs defined in the font
    #[inline]
    #[must_use]
    pub fn num_glyphs(&self) -> usize {
        self.num_glyphs as usize
    }
}
