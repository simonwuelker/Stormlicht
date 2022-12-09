use crate::ttf::{read_u16_at, read_u32_at, Readable, TTFParseError};

#[derive(Debug)]
pub enum PlatformID {
    Unicode,
    Mac,
    Reserved,
    Microsoft,
}

pub fn get_subtable_for_platform(data: &[u8], platform: PlatformID) -> Option<u32> {
    let version = read_u16_at(data, 0);
    let num_subtables = read_u16_at(data, 2);

    let target_id = match platform {
        PlatformID::Unicode => 0,
        PlatformID::Mac => 1,
        PlatformID::Reserved => 2,
        PlatformID::Microsoft => 3,
    };

    // using a linear search here - there are usually only 3 tables (TODO: verify)
    // so binary search really doesn't make a lot of sense
    for i in 0..num_subtables as usize {
        let platform_id = read_u16_at(data, 4 + i * 8);

        if platform_id == target_id {
            let _platform_specific_id = read_u16_at(data, 6 + i * 8);
            let offset = read_u32_at(data, 8 + i * 8);
            return Some(offset);
        }
    }
    None
}

pub fn search_for_subtable(data: &[u8], target_table_id: u16, search_range: usize) -> Option<u32> {
    if data.is_empty() {
        None
    } else {
        let index = (search_range / 2) * 8;
        let table_id = read_u16_at(data, index);
        println!(
            "table id: {table_id} at index {index}, search range {search_range} len {}",
            data.len()
        );
        if table_id == target_table_id {
            Some(read_u32_at(data, index + 4))
        } else if table_id < target_table_id {
            search_for_subtable(&data[index + 8..], target_table_id, search_range / 2)
        } else {
            search_for_subtable(&data[..index], target_table_id, search_range / 2)
        }
    }
}

pub struct Format4 {
    format: u16,
    length: u16,
    language: u16,
    segcount: usize,
    search_range: u16,
    entry_selector: u16,
    range_shift: u16,
    start_code: Vec<u16>,
    end_code: Vec<u16>,
    id_delta: Vec<u16>,
    id_range_offset: Vec<u16>,
    glyphs: Vec<u16>,
}

impl Readable for Format4 {
    fn read(data: &[u8]) -> Result<Self, TTFParseError> {
        // At least 16 bytes required, assuming segcount = 0
        if data.len() < 16 {
            return Err(TTFParseError::UnexpectedEOF);
        }

        let format = read_u16_at(data, 0);
        assert_eq!(format, 4);
        let length = read_u16_at(data, 2);

        let language = read_u16_at(data, 4);
        let segcount_x2 = read_u16_at(data, 6) as usize;
        let segcount = segcount_x2 / 2;
        let search_range = read_u16_at(data, 8);
        let entry_selector = read_u16_at(data, 10);
        let range_shift = read_u16_at(data, 12);

        let mut start_code = Vec::with_capacity(segcount);
        let mut end_code = Vec::with_capacity(segcount);
        let mut id_delta = Vec::with_capacity(segcount);
        let mut id_range_offset = Vec::with_capacity(segcount);

        // Byte layout looks like this:
        // End Code      : [u16; segcount]
        //                 < 2 byte padding>
        // Start Code    : [u16; segcount]
        // ID Delta      : [u16; segcount]
        // ID Range Offs : [u16; segcount]
        for i in 0..segcount {
            // arrays start at 14 bytes into the format4 struct
            let base = 14 + 2 * i;
            end_code.push(read_u16_at(data, base));

            // indexing is a bit awkward because there are two bytes of padding
            // after the end code (don't ask me why)
            start_code.push(read_u16_at(data, base + segcount_x2 + 2));
            id_delta.push(read_u16_at(data, base + 2 * segcount_x2 + 2));
            id_range_offset.push(read_u16_at(data, base + 3 * segcount_x2 + 2));
        }
        assert_eq!(*end_code.last().unwrap(), 0xFFFF);

        // Parse the glyph array, this takes up the rest of the table
        let current_position = 14 + segcount_x2 * 4;
        let num_glyphs = (length as usize - current_position) / 2;
        let mut glyphs = Vec::with_capacity(num_glyphs as usize);

        for i in 0..num_glyphs {
            glyphs.push(read_u16_at(data, current_position + 2 * i + 2));
        }

        Ok(Self {
            format: format,
            length: length,
            language: language,
            segcount: segcount,
            search_range: search_range,
            entry_selector: entry_selector,
            range_shift: range_shift,
            start_code: start_code,
            end_code: end_code,
            id_delta: id_delta,
            id_range_offset: id_range_offset,
            glyphs: glyphs,
        })
    }
}

impl Format4 {
    pub fn get_glyph_index(&self, codepoint: u16) -> Option<u16> {
        // TODO optimize. a lot.
        let mut index: usize = 0;
        while index < self.end_code.len() / 2 && self.end_code[index] < codepoint {
            index += 1;
        }
        assert!(self.end_code[index] > codepoint);

        if self.start_code[index] < codepoint {
            if self.id_range_offset[index] != 0 {
                let mut range_index = index + (self.id_range_offset[index] / 2) as usize;
                range_index += (codepoint - self.start_code[index]) as usize;
                // We don't do the unsafe spec pointer magic here!
                Some(self.glyphs[range_index - self.start_code.len()] + self.id_delta[index])
            } else {
                Some(codepoint + self.id_delta[index])
            }
        } else {
            None
        }
    }
}
