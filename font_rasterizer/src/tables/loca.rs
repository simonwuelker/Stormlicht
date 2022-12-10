use crate::ttf::{read_u16_at, read_u32_at, TTFParseError};

pub fn get_glyph_offset(
    data: &[u8],
    glyph_index: u32,
    loca_format: i16,
) -> Result<u32, TTFParseError> {
    if loca_format == 0 {
        // Short table, u16
        // Indexing is done in words
        // Also, the offset / 2 is stored (don't ask me why)
        Ok(read_u16_at(&data, glyph_index as usize * 2) as u32 * 2)
    } else {
        // Long table, u32
        // Indexing is done in bytes
        Ok(read_u32_at(&data, glyph_index as usize))
    }
}
