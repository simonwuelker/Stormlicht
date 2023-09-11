//! [CMAP](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6cmap.html) table implementation

use crate::ttf::{read_u16_at, read_u32_at};
use std::{cmp::Ordering, fmt};

/// Zero-cost wrapper around a `u16` for extra type safety.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GlyphID(u16);

impl GlyphID {
    /// The id of the replacement glyph
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

#[derive(Clone, Debug)]
pub struct Format4 {
    segments: Vec<Format4Segment>,
    glyph_id_data: Vec<u8>,
}

#[derive(Clone, Copy, Debug)]
struct Format4Segment {
    start_code: u16,
    end_code: u16,
    id_delta: u16,
    id_range_offset: u16,
}

impl Format4 {
    pub fn new(data: &[u8]) -> Self {
        // Byte layout looks like this:
        // Header        : 14 bytes
        // End Code      : [u16; segcount]
        //                 < 2 byte padding>
        // Start Code    : [u16; segcount]
        // ID Delta      : [u16; segcount]
        // ID Range Offs : [u16; segcount]
        // Glyph IDS     : remaining space

        let format = read_u16_at(data, 0);
        // FIXME: Propagate error, don't panic
        assert_eq!(format, 4, "not a format4 subtable");

        let length = read_u16_at(data, 2) as usize;
        let data = &data[..length];

        let segment_count_x2 = read_u16_at(data, 6) as usize;
        let segment_count = segment_count_x2 / 2;

        let mut segments = Vec::with_capacity(segment_count);
        for i in 0..segment_count {
            let end_code = read_u16_at(data, 14 + 2 * i);
            let start_code = read_u16_at(data, 16 + segment_count_x2 + 2 * i);
            let id_delta = read_u16_at(data, 16 + 2 * segment_count_x2 + 2 * i);
            let id_range_offset = read_u16_at(data, 16 + 3 * segment_count_x2 + 2 * i);

            segments.push(Format4Segment {
                start_code,
                end_code,
                id_delta,
                id_range_offset,
            });
        }

        let glyph_id_data = data[16 + 4 * segment_count_x2..].to_vec();
        Self {
            segments,
            glyph_id_data,
        }
    }

    pub fn get_glyph_id(&self, codepoint: u16) -> Option<GlyphID> {
        // Find the segment containing the glyph index
        let segment_index = self
            .segments()
            .binary_search_by(|segment| {
                if segment.start_code > codepoint {
                    Ordering::Greater
                } else if segment.end_code >= codepoint {
                    Ordering::Equal
                } else {
                    Ordering::Less
                }
            })
            .ok()?;

        let segment = self.segments()[segment_index];

        if segment.id_range_offset == 0 {
            let numeric_id = codepoint.wrapping_add(segment.id_delta);
            Some(GlyphID(numeric_id))
        } else {
            let delta = (codepoint - segment.start_code) * 2;

            // The specification abuses pointer magic here, which is kind of
            // clunky to replicate in a non-insane way.
            //
            // Basically, it computes a position which starts as a pointer into the segments
            // id range offset value, thats then modified until it points somewhere into
            // the glyph id bytes.
            //
            // Since the glyph id range values are stored right before glyph id values, we simply
            // store an index and subtract the length of the glyph id range array, such that we end
            // up with an index into the glyph ids
            //
            // The fact that this trickery worked first try is suspicious...
            let mut pos = segment_index as u16 * 2;
            pos = pos.wrapping_add(delta);
            pos = pos.wrapping_add(segment.id_range_offset);

            let pos = pos as usize - self.segments().len() * 2;

            let glyph_id = read_u16_at(&self.glyph_id_data, pos).wrapping_add(segment.id_delta);
            Some(GlyphID(glyph_id))
        }
    }

    #[inline]
    #[must_use]
    fn segments(&self) -> &[Format4Segment] {
        &self.segments
    }

    /// Call `f` for every codepoint defined in the font
    pub fn codepoints<F: FnMut(u16)>(&self, mut f: F) {
        for segment in self.segments() {
            // Indicates the final segment
            if segment.start_code == 0xFFFF && segment.end_code == 0xFFFF {
                break;
            }

            for codepoint in segment.start_code..=segment.end_code {
                f(codepoint)
            }
        }
    }
}
