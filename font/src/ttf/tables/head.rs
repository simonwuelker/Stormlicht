//! [Head](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6head.html) table implementation

use crate::ttf::{read_i16_at, read_u16_at};
use std::fmt;

pub struct HeadTable<'a>(&'a [u8]);

impl<'a> HeadTable<'a> {
    pub fn new(data: &'a [u8], offset: usize) -> Self {
        Self(&data[offset..][..54])
    }

    pub fn units_per_em(&self) -> u16 {
        read_u16_at(self.0, 18)
    }

    /// The minimum x value that can be encountered while
    /// rendering a glyph from this font, in `FUnits`.
    pub fn min_x(&self) -> i16 {
        read_i16_at(self.0, 36)
    }

    /// The minimum y value that can be encountered while
    /// rendering a glyph from this font, in `FUnits`.
    pub fn min_y(&self) -> i16 {
        read_i16_at(self.0, 38)
    }

    /// The maximum x value that can be encountered while
    /// rendering a glyph from this font, in `FUnits`.
    pub fn max_x(&self) -> i16 {
        read_i16_at(self.0, 40)
    }

    /// The maximum y value that can be encountered while
    /// rendering a glyph from this font, in `FUnits`.
    pub fn max_y(&self) -> i16 {
        read_i16_at(self.0, 42)
    }

    /// Get the format of the [Loca Table](crate::ttf::tables::loca::LocaTable).
    ///
    /// If this is `0`, the loca table is in `short` format.
    /// Otherwise, it is in `long` format.
    pub fn index_to_loc_format(&self) -> i16 {
        read_i16_at(self.0, 50)
    }
}

impl<'a> fmt::Debug for HeadTable<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Head Table")
            .field("index_to_loc_format", &self.index_to_loc_format())
            .finish()
    }
}
