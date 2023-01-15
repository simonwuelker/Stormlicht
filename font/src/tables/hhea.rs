//! Horizontal Header Table
//! 
//! Mostly just contains information for the [hmtx](crate::tables::hmtx) table.

use crate::ttf::read_u16_at;

pub struct HHEATable<'a>(&'a [u8]);

impl<'a> HHEATable<'a> {
    pub fn new(data: &'a [u8], offset: usize) -> Self {
        Self(&data[offset..][..288])
    }

    pub fn num_of_long_hor_metrics(&self) -> usize {
        read_u16_at(&self.0, 272) as usize
    }
}
