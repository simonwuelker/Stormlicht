use crate::ttf::{read_i16_at, Readable, TTFParseError};
use std::fmt;

pub struct Head<'a>(&'a [u8]);

impl<'a> Head<'a> {
    pub fn new(data: &'a [u8], offset: usize) -> Self {
        Self(&data[offset..][..54])
    }

    pub fn index_to_loc_format(&self) -> i16 {
        read_i16_at(self.0, 50)
    }
}

impl<'a> fmt::Debug for Head<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Head Table")
            .field("index_to_loc_format", &self.index_to_loc_format())
            .finish()
    }
}
