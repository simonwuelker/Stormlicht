use std::collections::HashMap;

use crate::ttf;

const DEFAULT_FONT: &[u8; 168644] =
    include_bytes!("../../downloads/fonts/roboto/Roboto-Medium.ttf");

pub struct Outline {
    points: Vec<(i32, i32)>,
}

#[derive(Clone, Copy, Debug)]
pub struct BoundingBox {
    min_x: i32,
    max_x: i32,
    min_y: i32,
    max_y: i32,
}

#[derive(Clone, Copy, Debug)]
pub struct LayoutInfo {
    advance_width: usize,
    advance_height: usize,
}
pub struct Glyph {
    outlines: Vec<Outline>,
    layout_info: LayoutInfo,
}

pub struct Font {
    name: Option<String>,
    units_per_em: f32,
    glyphs: Vec<Glyph>,
    glyph_indices: HashMap<u16, u16>,
}

impl Default for Font {
    fn default() -> Self {
        Self::from_bytes(DEFAULT_FONT)
    }
}

impl Font {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let ttf_font = ttf::Font::new(bytes).unwrap();
        let name = ttf_font.name().get_font_name();

        let num_glyphs = ttf_font.num_glyphs();
        let glyphs = Vec::with_capacity(num_glyphs);
        let mut glyph_indices = HashMap::with_capacity(num_glyphs);

        let format_4 = ttf_font.format_4();
        format_4.codepoints(|codepoint| {
            let glyph_index = format_4.get_glyph_index(codepoint).unwrap_or_default();
            glyph_indices.insert(codepoint, glyph_index);
        });

        Self {
            name: name,
            units_per_em: ttf_font.units_per_em() as f32,
            glyphs: glyphs,
            glyph_indices: glyph_indices,
        }
    }

    /// Return the number of coordinate points per font size unit.
    /// This value is used to scale fonts, ie. when you render a font with
    /// size `17px`, one `em` equals `17px`.
    ///
    /// Note that this value does not constrain the size of individual glyphs.
    /// A glyph may have a size larger than `1em`.
    pub fn units_per_em(&self) -> f32 {
        self.units_per_em
    }

    /// Get the number of glyphs defined by the font
    pub fn num_glyphs(&self) -> usize {
        self.glyphs.len()
    }

    pub fn name(&self) -> Option<&String> {
        self.name.as_ref()
    }

    /// Find the glyph for a given character.
    /// If the font does not contain the character,
    /// the replacement glyph (index 0) is returned instead.
    pub fn get_glyph(&self, codepoint: u16) -> &Glyph {
        self.get_indexed(self.get_glyph_index(codepoint))
    }

    pub fn get_glyph_index(&self, codepoint: u16) -> usize {
        self.glyph_indices
            .get(&codepoint)
            .copied()
            .unwrap_or_default() as usize
    }

    /// Get a glyph by it's index.
    /// If possible, you should prefer to use [Self::get_glyph] instead.
    fn get_indexed(&self, index: usize) -> &Glyph {
        &self.glyphs[index]
    }
}
