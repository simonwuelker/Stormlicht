//! [Horizontal Metrics](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6hmtx.html) table

use crate::ttf::{read_i16_at, read_u16_at};

use super::cmap::GlyphID;

pub struct HMTXTable<'a>(&'a [u8]);

impl<'a> HMTXTable<'a> {
    pub fn new(data: &'a [u8], offset: usize, num_of_long_hor_metrics: usize) -> Self {
        Self(&data[offset..][..num_of_long_hor_metrics * 4])
    }

    pub fn get_metric_for(&self, glyph_id: GlyphID) -> LongHorMetric {
        LongHorMetric {
            advance_width: read_u16_at(self.0, glyph_id.numeric() as usize * 4),
            left_side_bearing: read_i16_at(self.0, glyph_id.numeric() as usize * 4 + 2),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct LongHorMetric {
    advance_width: u16,
    left_side_bearing: i16,
}

impl LongHorMetric {
    pub fn advance_width(&self) -> u16 {
        self.advance_width
    }

    pub fn left_side_bearing(&self) -> i16 {
        self.left_side_bearing
    }
}
