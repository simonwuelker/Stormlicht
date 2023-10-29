//! [Loca](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6loca.html) table implementation.

use super::{cmap::GlyphID, head::LocaTableFormat};

#[derive(Clone, Debug)]
pub struct LocaTable {
    glyph_locations: Vec<GlyphLocation>,
}

#[derive(Clone, Copy, Debug)]
pub struct GlyphLocation {
    pub offset: u32,
    pub length: u32,
}

impl LocaTable {
    pub fn new(data: &[u8], format: LocaTableFormat, num_glyphs: usize) -> Self {
        let glyph_locations = match format {
            LocaTableFormat::Short => {
                // Short table, u16
                // Indexing is done in words
                data.array_chunks::<2>()
                    .map_windows(|[&offset, &next_offset]| {
                        let offset = u16::from_be_bytes(offset) as u32 * 2;
                        let next_offset = u16::from_be_bytes(next_offset) as u32 * 2;

                        GlyphLocation {
                            offset,
                            length: next_offset - offset,
                        }
                    })
                    .take(num_glyphs)
                    .collect()
            },
            LocaTableFormat::Long => {
                // Long table, u32
                // Indexing is done in bytes
                data.array_chunks::<4>()
                    .map_windows(|[&offset, &next_offset]| {
                        let offset = u32::from_be_bytes(offset);
                        let next_offset = u32::from_be_bytes(next_offset);

                        GlyphLocation {
                            offset,
                            length: next_offset - offset,
                        }
                    })
                    .take(num_glyphs)
                    .collect()
            },
        };

        Self { glyph_locations }
    }

    pub fn get_glyph_offset(&self, glyph_index: GlyphID) -> GlyphLocation {
        self.glyph_locations[glyph_index.numeric() as usize]
    }
}
