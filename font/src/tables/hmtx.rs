use crate::ttf::{read_i16_at, read_u16_at};

pub struct HMTXTable<'a>(&'a [u8]);

impl<'a> HMTXTable<'a> {
    pub fn new(data: &'a [u8], offset: usize, num_of_long_hor_metrics: usize) -> Self {
        println!("num of long hor metrics {num_of_long_hor_metrics}");
        Self(&data[offset..][..num_of_long_hor_metrics * 4])
    }

    pub fn get_metric_for(&self, glyph_id: u16) -> LongHorMetric {
        LongHorMetric {
            advance_width: read_u16_at(self.0, glyph_id as usize * 4),
            left_side_bearing: read_i16_at(self.0, glyph_id as usize * 4 + 2),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LongHorMetric {
    advance_width: u16,
    left_side_bearing: i16,
}

impl LongHorMetric {
    pub fn advance_width(&self) -> usize {
        self.advance_width as usize
    }

    pub fn left_side_bearing(&self) -> i16 {
        self.left_side_bearing
    }
}
