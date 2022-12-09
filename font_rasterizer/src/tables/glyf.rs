use crate::ttf::{read_i16_at, read_u16_at, TTFParseError};
use std::fmt;

pub struct GlyphOutlineTable<'a>(&'a [u8]);

impl<'a> GlyphOutlineTable<'a> {
    pub fn new(data: &'a [u8], offset: usize, length: usize) -> Self {
        Self(&data[offset..][..length])
    }

    pub fn get_glyph_outline(&self, glyph_index: u32) -> GlyphOutline<'a> {
        let data = &self.0[glyph_index as usize..];
        let num_contours_maybe_negative = read_i16_at(data, 0);
        assert!(num_contours_maybe_negative >= 0, "compound glyphs are not supported");
        let num_contours = num_contours_maybe_negative as usize;

        // 8 bytes of coordinates follow
        let last_index = read_u16_at(data, 10 + num_contours * 2 - 2) as usize;
        let instruction_length = read_u16_at(data, 10 + num_contours * 2) as usize;

        // Memory map is like this:
        // num contours          : i16
        // min x                 : i16
        // min y                 : i16
        // max x                 : i16
        // max y                 : i16
        // end points of contours: [u16; num contours]
        // instruction length    : u16
        // instructions          : [u8; instruction length]
        // flags                 : [u8; last value in "end points of contours" + 1]
        // x coords              : [u16; last value in "end points of contours" + 1]
        // y coords              : [u16; last value in "end points of contours" + 1]
        let total_size = 10 + num_contours * 2 + 2 + instruction_length + (last_index + 1) * 5;

        GlyphOutline(&data[..total_size])
    }
}

/// Constructed via the [GlyphOutlineTable].
pub struct GlyphOutline<'a>(&'a [u8]);

impl<'a> GlyphOutline<'a> {
    pub fn is_simple(&self) -> bool {
        read_i16_at(self.0, 0) >= 0
    }

    /// If the Glyph is a simple glyph, this is the number of contours.
    /// Don't call this if the glyph is not simple.
    pub fn num_contours(&self) -> usize {
        assert!(self.is_simple());
        read_i16_at(self.0, 0) as usize
    }

    pub fn min_x(&self) -> i16 {
        read_i16_at(self.0, 2)
    }

    pub fn min_y(&self) -> i16 {
        read_i16_at(self.0, 4)
    }

    pub fn max_x(&self) -> i16 {
        read_i16_at(self.0, 6)
    }

    pub fn max_y(&self) -> i16 {
        read_i16_at(self.0, 8)
    }

    pub fn instruction_length(&self) -> u16 {
        read_u16_at(self.0, 10 + self.num_contours() * 2)
    }
}

// TODO: this panics if the glyf is not simple :^(
impl<'a> fmt::Debug for GlyphOutline<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Glyf Table")
            .field("is_simple", &self.is_simple())
            .field("num_contours", &self.num_contours())
            .field("min_x", &self.min_x())
            .field("min_y", &self.min_y())
            .field("max_x", &self.max_x())
            .field("max_y", &self.max_y())
            .field("instruction_length", &self.instruction_length())
            .finish()
    }
}

// #[derive(Debug)]
// pub struct GlyfFlag(u8);
// 
// impl GlyfFlag {
//     const POINT_ON_CURVE: u8 = 1;
//     const SHORT_X: u8 = 2;
//     const SHORT_Y: u8 = 4;
//     const REPEAT: u8 = 8;
// 
//     pub fn is_on_curve(&self) -> bool {
//         self.0 & Self::POINT_ON_CURVE != 0
//     }
// 
//     pub fn is_short_x(&self) -> bool {
//         self.0 & Self::SHORT_X != 0
//     }
// 
//     pub fn is_short_y(&self) -> bool {
//         self.0 & Self::SHORT_Y != 0
//     }
// 
//     pub fn repeat(&self) -> bool {
//         self.0 & Self::REPEAT != 0
//     }
// }
