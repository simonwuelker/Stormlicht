use crate::ttf::{read_u16_at, read_u32_at, TTFParseError};

pub struct LocaTable<'a>(&'a [u8]);

impl<'a> LocaTable<'a> {
    pub fn new(data: &'a [u8], offset: usize) -> Self {
        Self(&data[offset..])
    }

    pub fn get_glyph_offset(
        &self,
        glyph_index: u16,
        loca_format: i16,
    ) -> Result<u32, TTFParseError> {
        if loca_format == 0 {
            // Short table, u16
            // Indexing is done in words
            // Also, the offset / 2 is stored (don't ask me why)
            Ok(read_u16_at(self.0, glyph_index as usize * 2) as u32 * 2)
        } else {
            // Long table, u32
            // Indexing is done in bytes
            Ok(read_u32_at(self.0, glyph_index as usize))
        }
    }
}
