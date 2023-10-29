//! [Head](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6head.html) table implementation

use crate::ttf::{read_i16_at, read_u16_at};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LocaTableFormat {
    Short,
    Long,
}

#[derive(Clone, Copy, Debug)]
pub struct HeadTable {
    units_per_em: u16,

    /// The minimum x value that can be encountered while
    /// rendering a glyph from this font, in `FUnits`.
    min_x: i16,

    /// The minimum y value that can be encountered while
    /// rendering a glyph from this font, in `FUnits`.
    min_y: i16,

    /// The maximum x value that can be encountered while
    /// rendering a glyph from this font, in `FUnits`.
    max_x: i16,

    /// The maximum y value that can be encountered while
    /// rendering a glyph from this font, in `FUnits`.
    max_y: i16,

    loca_table_format: LocaTableFormat,
}

impl HeadTable {
    pub fn new(data: &[u8], offset: usize) -> Self {
        let data = &data[offset..];
        let loca_table_format = if read_i16_at(data, 50) == 0 {
            LocaTableFormat::Short
        } else {
            LocaTableFormat::Long
        };

        Self {
            units_per_em: read_u16_at(data, 18),
            min_x: read_i16_at(data, 36),
            min_y: read_i16_at(data, 38),
            max_x: read_i16_at(data, 40),
            max_y: read_i16_at(data, 42),
            loca_table_format,
        }
    }

    #[inline]
    #[must_use]
    pub fn units_per_em(&self) -> u16 {
        self.units_per_em
    }

    #[inline]
    #[must_use]
    pub fn min_x(&self) -> i16 {
        self.min_x
    }

    #[inline]
    #[must_use]
    pub fn min_y(&self) -> i16 {
        self.min_y
    }

    #[inline]
    #[must_use]
    pub fn max_x(&self) -> i16 {
        self.max_x
    }

    #[inline]
    #[must_use]
    pub fn max_y(&self) -> i16 {
        self.max_y
    }

    /// Get the format of the [Loca Table](crate::ttf::tables::loca::LocaTable).
    #[inline]
    #[must_use]
    pub fn loca_table_format(&self) -> LocaTableFormat {
        self.loca_table_format
    }
}
