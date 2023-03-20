//! <https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6vmtx.html>
use crate::ttf::{read_i16_at, read_u16_at};

/// Vertical Metrics Table
pub struct VMTXTable<'a>(&'a [u8]);

impl<'a> VMTXTable<'a> {
    pub fn new(data: &'a [u8], offset: usize, num_of_long_hor_metrics: usize) -> Self {
        Self(&data[offset..][..num_of_long_hor_metrics * 4])
    }

    pub fn get_metric_for(&self, glyph_id: u16) -> LongVerticalMetric {
        LongVerticalMetric {
            advance_height: read_u16_at(self.0, glyph_id as usize * 4),
            top_side_bearing: read_i16_at(self.0, glyph_id as usize * 4 + 2),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LongVerticalMetric {
    advance_height: u16,
    top_side_bearing: i16,
}

impl LongVerticalMetric {
    pub fn advance_height(&self) -> usize {
        self.advance_height as usize
    }

    pub fn top_side_bearing(&self) -> i16 {
        self.top_side_bearing
    }
}
