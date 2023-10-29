//! [Horizontal Metrics](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6hmtx.html) table

use crate::ttf::{read_i16_at, read_u16_at};

use super::cmap::GlyphID;

#[derive(Clone, Debug)]
pub struct HMTXTable {
    long_hor_metrics: Vec<LongHorMetric>,
}

impl HMTXTable {
    pub fn new(data: &[u8], num_of_long_hor_metrics: usize) -> Self {
        let long_hor_metrics = data
            .array_chunks::<4>()
            .take(num_of_long_hor_metrics)
            .map(|metric_data| LongHorMetric {
                advance_width: read_u16_at(metric_data, 0),
                left_side_bearing: read_i16_at(metric_data, 2),
            })
            .collect();

        Self { long_hor_metrics }
    }

    #[inline]
    #[must_use]
    pub fn get_metric_for(&self, glyph_id: GlyphID) -> LongHorMetric {
        self.long_hor_metrics[glyph_id.numeric() as usize]
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct LongHorMetric {
    advance_width: u16,
    left_side_bearing: i16,
}

impl LongHorMetric {
    #[inline]
    #[must_use]
    pub fn advance_width(&self) -> u16 {
        self.advance_width
    }

    #[inline]
    #[must_use]
    pub fn left_side_bearing(&self) -> i16 {
        self.left_side_bearing
    }
}
