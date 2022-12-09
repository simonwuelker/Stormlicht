use crate::ttf::{read_u16_at, read_u32_at, TTFParseError};
use std::fmt;

#[derive(Debug)]
pub enum PlatformID {
    Unicode,
    Mac,
    Reserved,
    Microsoft,
}

pub struct CMAPTable<'a>(&'a [u8]);

impl<'a> CMAPTable<'a> {
    /// You can technically construct a CMAPTable without calling this method.
    /// But using this will protect you from out of bound reads
    pub fn new(data: &'a[u8], offset: usize) -> Self {
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

    pub fn get_subtable_for_platform(&self, platform: PlatformID) -> Option<usize> {
        let target_id = match platform {
            PlatformID::Unicode => 0,
            PlatformID::Mac => 1,
            PlatformID::Reserved => 2,
            PlatformID::Microsoft => 3,
        };

        // using a linear search here - there are usually only 3 tables (TODO: verify)
        // so binary search really doesn't make a lot of sense
        for i in 0..self.num_subtables() as usize {
            let subtable = self.get_nth_subtable(i);
            let platform_id = read_u16_at(self.0, 4 + i * 8);

            if subtable.platform_id() == target_id {
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

    pub fn platform_id(&self) -> u16 {
        read_u16_at(self.0, 0)
    }

    pub fn platform_specific_id(&self) -> u16 {
        read_u16_at(self.0, 2)
    }

    pub fn offset(&self) -> usize {
        read_u32_at(self.0, 4) as usize
    }
}

impl<'a> fmt::Debug for CMAPSubTable<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CMAP Subtable")
            .field("platform_id", &self.platform_id())
            .field("platform_specific_id", &self.platform_specific_id())
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

    pub fn length(&self) -> u16 {
        read_u16_at(self.0, 2)
    }

    pub fn segcount_x2(&self) -> usize {
        read_u16_at(self.0, 6) as usize
    }

    pub fn segcount(&self) -> usize {
        self.segcount_x2() / 2
    }

    pub fn get_start_code(&self, index: usize) -> u16 {
        read_u16_at(self.0, self.start_code_start() + index * 2)
    }

    pub fn get_end_code(&self, index: usize) -> u16 {
        read_u16_at(self.0, self.end_code_start() + index * 2)
    }

    pub fn get_id_delta(&self, index: usize) -> u16 {
        read_u16_at(self.0, self.id_delta_start() + index * 2)
    }

    pub fn get_id_range_offset(&self, index: usize) -> u16 {
        read_u16_at(self.0, self.id_range_offset_start() + index * 2)
    }

    pub fn get_glyph(&self, index: usize) -> u16 {
        read_u16_at(self.0, self.glyph_ids_start() + index * 2)
    }

    pub fn get_glyph_index(&self, codepoint: u16) -> Option<u16> {
        // TODO optimize. a lot.
        let mut index: usize = 0;
        while index < self.segcount() && self.get_end_code(index) < codepoint {
            index += 1;
        }
        assert!(self.get_end_code(index) > codepoint);

        if self.get_start_code(index) < codepoint {
            if self.get_id_range_offset(index) != 0 {
                let mut range_index = index + (self.get_id_range_offset(index) / 2) as usize;
                range_index += (codepoint - self.get_start_code(index)) as usize;
                // We don't do the unsafe spec pointer magic here!
                Some(self.get_id_range_offset(range_index) + self.get_id_delta(index))
            } else {
                Some(codepoint + self.get_id_delta(index))
            }
        } else {
            None
        }
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
}

impl<'a> fmt::Debug for Format4<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Format 4")
            .field("length", &self.length())
            .field("segcount_x2", &self.segcount_x2())
            .finish()
    }
}
