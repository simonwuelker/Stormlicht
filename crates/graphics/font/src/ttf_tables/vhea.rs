//! [Vertical Header](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6hhea.html) Table
//!
//! Mostly just contains information for the [vmtx](super::vmtx) table.

use crate::ttf::read_u16_at;

pub struct VHEATable<'a>(&'a [u8]);

impl<'a> VHEATable<'a> {
    pub fn new(data: &'a [u8], offset: usize) -> Self {
        Self(&data[offset..][..36])
    }

    pub fn num_of_long_vertical_metrics(&self) -> usize {
        read_u16_at(self.0, 34) as usize
    }
}
