//! [CMAP](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6cmap.html) table implementation

use crate::ttf::{read_u16_at, read_u32_at};
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Zero-cost wrapper around a `u16` for extra type safety.
pub struct GlyphID(u16);

impl GlyphID {
    pub const REPLACEMENT: Self = Self(0);

    #[inline]
    pub fn new(value: u16) -> Self {
        Self(value)
    }

    #[inline]
    pub fn numeric(self) -> u16 {
        self.0
    }
}

impl From<GlyphID> for u16 {
    fn from(value: GlyphID) -> Self {
        value.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlatformID {
    Unicode(UnicodePlatformSpecificID),
    Mac,
    Reserved,
    Microsoft(WindowsPlatformSpecificID),
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnicodePlatformSpecificID {
    Version1_0,
    Version1_1,
    Iso10646_1993SemanticDeprecated,
    Unicode2_0OrLaterBmpOnly,
    Unicode2_0OrLater,
    UnicodeVariationSequences,
    LastResort,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowsPlatformSpecificID {
    Symbol,
    UnicodeBmpOnly,
    ShiftJis,
    Prc,
    BigFive,
    Johab,
    UnicodeUcs4,
    Unknown,
}

impl From<(u16, u16)> for PlatformID {
    fn from(value: (u16, u16)) -> Self {
        match value.0 {
            0 => Self::Unicode(value.1.into()),
            1 => Self::Mac,
            2 => Self::Reserved,
            3 => Self::Microsoft(value.1.into()),
            _ => Self::Unknown,
        }
    }
}

impl From<u16> for UnicodePlatformSpecificID {
    fn from(value: u16) -> Self {
        match value {
            0 => Self::Version1_0,
            1 => Self::Version1_1,
            2 => Self::Iso10646_1993SemanticDeprecated,
            3 => Self::Unicode2_0OrLaterBmpOnly,
            4 => Self::Unicode2_0OrLater,
            5 => Self::UnicodeVariationSequences,
            6 => Self::LastResort,
            _ => Self::Unknown,
        }
    }
}

impl From<u16> for WindowsPlatformSpecificID {
    fn from(value: u16) -> Self {
        match value {
            0 => Self::Symbol,
            1 => Self::UnicodeBmpOnly,
            2 => Self::ShiftJis,
            3 => Self::Prc,
            4 => Self::BigFive,
            5 => Self::Johab,
            10 => Self::UnicodeUcs4,
            _ => Self::Unknown,
        }
    }
}

pub struct CMAPTable<'a>(&'a [u8]);

impl<'a> CMAPTable<'a> {
    /// You can technically construct a CMAPTable without calling this method.
    /// But using this will protect you from out of bound reads
    pub fn new(data: &'a [u8], offset: usize) -> Self {
        let num_subtables = read_u16_at(&data[offset..], 2) as usize;
        // 4 bytes header + 8 bytes per table
        Self(&data[offset..][..4 + num_subtables * 8])
    }

    pub fn version(&self) -> u16 {
        read_u16_at(self.0, 0)
    }

    pub fn num_subtables(&self) -> usize {
        read_u16_at(self.0, 2) as usize
    }

    pub fn get_nth_subtable(&self, n: usize) -> CMAPSubTable<'a> {
        assert!(n < self.num_subtables());
        // 4 bytes header + 8 bytes for each subtable
        CMAPSubTable::new(self.0, 4 + n * 8)
    }

    pub fn get_unicode_table(&self) -> Option<usize> {
        // using a linear search here - there are usually only 3 tables (TODO: verify)
        // so binary search really doesn't make a lot of sense
        for i in 0..self.num_subtables() {
            let subtable = self.get_nth_subtable(i);
            let platform_id = subtable.platform_id();

            if matches!(
                platform_id,
                PlatformID::Unicode(_)
                    | PlatformID::Microsoft(WindowsPlatformSpecificID::UnicodeBmpOnly)
                    | PlatformID::Microsoft(WindowsPlatformSpecificID::UnicodeUcs4)
            ) {
                return Some(subtable.offset());
            }
        }
        None
    }
}

impl<'a> fmt::Debug for CMAPTable<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CMAP Table")
            .field("version", &self.version())
            .field("num_subtables", &self.num_subtables())
            .finish()
    }
}

pub struct CMAPSubTable<'a>(&'a [u8]);

impl<'a> CMAPSubTable<'a> {
    pub fn new(data: &'a [u8], offset: usize) -> Self {
        Self(&data[offset..][..8])
    }

    pub fn platform_id(&self) -> PlatformID {
        let platform_id = read_u16_at(self.0, 0);
        let platform_specific_id = read_u16_at(self.0, 2);
        (platform_id, platform_specific_id).into()
    }

    pub fn offset(&self) -> usize {
        read_u32_at(self.0, 4) as usize
    }
}

impl<'a> fmt::Debug for CMAPSubTable<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CMAP Subtable")
            .field("platform_id", &self.platform_id())
            .field("offset", &self.offset())
            .finish()
    }
}

pub struct Format4<'a>(&'a [u8]);

impl<'a> Format4<'a> {
    pub fn new(data: &'a [u8], offset: usize) -> Self {
        let our_data = &data[offset..];
        let format = read_u16_at(our_data, 0);
        assert_eq!(format, 4, "not a format4 subtable");

        let length = read_u16_at(our_data, 2) as usize;

        // Byte layout looks like this:
        // Header        : 14 bytes
        // End Code      : [u16; segcount]
        //                 < 2 byte padding>
        // Start Code    : [u16; segcount]
        // ID Delta      : [u16; segcount]
        // ID Range Offs : [u16; segcount]
        // Glyph IDS     : remaining space
        Self(&our_data[..length])
    }

    fn length(&self) -> u16 {
        read_u16_at(self.0, 2)
    }

    fn segcount_x2(&self) -> usize {
        read_u16_at(self.0, 6) as usize
    }

    /// Get the number of segments in the table
    fn segcount(&self) -> usize {
        self.segcount_x2() / 2
    }

    /// Get the start code for a given segment
    fn get_start_code(&self, index: usize) -> u16 {
        assert!(index < self.segcount());
        read_u16_at(self.0, self.start_code_start() + index * 2)
    }

    /// Get the end code for a given segment
    pub fn get_end_code(&self, index: usize) -> u16 {
        assert!(index < self.segcount());
        read_u16_at(self.0, self.end_code_start() + index * 2)
    }

    fn get_id_delta(&self, index: usize) -> u16 {
        assert!(index < self.segcount());
        read_u16_at(self.0, self.id_delta_start() + index * 2)
    }

    fn get_id_range_offset(&self, index: usize) -> u16 {
        assert!(index < self.segcount());
        read_u16_at(self.0, self.id_range_offset_start() + index * 2)
    }

    pub fn get_glyph(&self, index: usize) -> u16 {
        read_u16_at(self.0, self.glyph_ids_start() + index * 2)
    }

    pub fn get_glyph_id(&self, codepoint: u16) -> Option<GlyphID> {
        // Find the segment containing the glyph index
        // using binary search
        let mut start = 0;
        let mut end = self.segcount();

        while end > start {
            let index = (start + end) / 2;
            let start_code = self.get_start_code(index);

            if start_code > codepoint {
                // The correct segment must have a lower index
                end = index;
            } else {
                let end_code = self.get_end_code(index);
                if end_code >= codepoint {
                    // We have found the correct segment
                    let id_delta = self.get_id_delta(index);
                    let id_range_offset = self.get_id_range_offset(index);

                    if id_range_offset == 0 {
                        return Some(GlyphID(codepoint.wrapping_add(id_delta)));
                    } else {
                        let delta = (codepoint - start_code) * 2;

                        let mut pos = (self.id_range_offset_start() + index * 2) as u16;
                        pos = pos.wrapping_add(delta);
                        pos = pos.wrapping_add(id_range_offset);

                        let glyph_id = read_u16_at(self.0, pos as usize);
                        return Some(GlyphID(glyph_id.wrapping_add(id_delta)));
                    }
                } else {
                    // The correct segment must have a higher index
                    start = index + 1;
                }
            }
        }

        // missing glyph
        None
    }

    fn end_code_start(&self) -> usize {
        14
    }

    fn start_code_start(&self) -> usize {
        self.end_code_start() + self.segcount_x2() + 2 // two bytes of padding
    }

    fn id_delta_start(&self) -> usize {
        self.start_code_start() + self.segcount_x2()
    }

    fn id_range_offset_start(&self) -> usize {
        self.id_delta_start() + self.segcount_x2()
    }

    fn glyph_ids_start(&self) -> usize {
        self.id_range_offset_start() + self.segcount_x2()
    }

    /// Call `f` for every codepoint defined in the font
    pub fn codepoints<F: FnMut(u16)>(&self, mut f: F) {
        for segment_index in 0..self.segcount() {
            let start = self.get_start_code(segment_index);
            let end = self.get_end_code(segment_index);

            // Indicates the final segment
            if start == end && end == 0xFFFF {
                break;
            }

            for codepoint in start..=end {
                f(codepoint)
            }
        }
    }
}

impl<'a> fmt::Debug for Format4<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Format 4")
            .field("length", &self.length())
            .field("segcount_x2", &self.segcount_x2())
            .finish()
    }
}