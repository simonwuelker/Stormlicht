//! [Loca](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6loca.html) table implementation.

use crate::ttf::{read_u16_at, read_u32_at};

use super::{cmap::GlyphID, head::LocaTableFormat};

pub struct LocaTable<'a> {
    data: &'a [u8],
    is_short: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct GlyphLocation {
    pub offset: u32,
    pub length: u32,
}

impl<'a> LocaTable<'a> {
    pub fn new(data: &'a [u8], offset: usize, format: LocaTableFormat) -> Self {
        Self {
            data: &data[offset..],
            is_short: format == LocaTableFormat::Short,
        }
    }

    pub fn get_glyph_offset(&self, glyph_index: GlyphID) -> GlyphLocation {
        if self.is_short {
            // Short table, u16
            // Indexing is done in words
            // Also, the offset / 2 is stored (don't ask me why)
            let offset = read_u16_at(self.data, glyph_index.numeric() as usize * 2) as u32 * 2;
            let offset_of_next_glyph =
                read_u16_at(self.data, (glyph_index.numeric() + 1) as usize * 2) as u32 * 2;
            GlyphLocation {
                offset,
                length: offset_of_next_glyph - offset,
            }
        } else {
            // Long table, u32
            // Indexing is done in bytes
            let offset = read_u32_at(self.data, glyph_index.numeric() as usize * 4);
            let offset_of_next_glyph =
                read_u32_at(self.data, (glyph_index.numeric() + 1) as usize * 4);
            GlyphLocation {
                offset,
                length: offset_of_next_glyph - offset,
            }
        }
    }
}
