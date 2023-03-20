//! [Loca](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6loca.html) table implementation.

use crate::ttf::{read_u16_at, read_u32_at};

pub struct LocaTable<'a> {
    data: &'a [u8],
    is_short: bool,
}

impl<'a> LocaTable<'a> {
    pub fn new(data: &'a [u8], offset: usize, format: i16) -> Self {
        Self {
            data: &data[offset..],
            is_short: format == 0,
        }
    }

    pub fn get_glyph_offset(&self, glyph_index: u16) -> u32 {
        if self.is_short {
            // Short table, u16
            // Indexing is done in words
            // Also, the offset / 2 is stored (don't ask me why)
            read_u16_at(self.data, (glyph_index) as usize * 2) as u32 * 2
        } else {
            // Long table, u32
            // Indexing is done in bytes
            read_u32_at(self.data, glyph_index as usize)
        }
    }
}
