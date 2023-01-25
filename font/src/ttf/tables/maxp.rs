//! [MaxP](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6maxp.html) table implementation.
use crate::ttf::read_u16_at;

pub struct MaxPTable<'a>(&'a [u8]);

impl<'a> MaxPTable<'a> {
    pub fn new(data: &'a [u8], offset: usize) -> Self {
        Self(&data[offset..])
    }

    /// Get the number of glyphs defined in the font
    pub fn num_glyphs(&self) -> usize {
        read_u16_at(self.0, 4) as usize
    }
}
