use crate::ttf::{TTFParseError, read_u16, read_u32};
use std::io::Read;

#[derive(Debug)]
pub struct CMAPSubTable {
    pub platform_id: CMAPPlatformID,
    pub platform_specific_id: u16, 
    pub offset: u32,

}

#[derive(Debug)]
pub enum CMAPPlatformID {
    Unicode,
    Mac,
    Reserved,
    Microsoft,
}

pub fn read_subtables<R: Read>(data: &mut R) -> Result<Vec<CMAPSubTable>, TTFParseError> {
    let version = read_u16(data)?;
    let n_subtables = read_u16(data)?;
    let mut tables = Vec::with_capacity(n_subtables as usize);

    for _ in 0..n_subtables {
        let platform_id = match read_u16(data)? {
            0 => CMAPPlatformID::Unicode,
            1 => CMAPPlatformID::Mac,
            2 => CMAPPlatformID::Reserved,
            3 => CMAPPlatformID::Microsoft,
            _ => return Err(TTFParseError::UnsupportedFormat),
        };

        tables.push(CMAPSubTable {
            platform_id: platform_id,
            platform_specific_id: read_u16(data)?,
            offset: read_u32(data)?,
        });
    }
    Ok(tables)
}

pub fn read_format_4<R: Read>(data: &mut R) -> Result<Format4, TTFParseError> {
    // format has already been consumed at this point
    let length = read_u16(data)?;
    let language = read_u16(data)?;
    let segcount_x2 = read_u16(data)?;
    let _search_range = read_u16(data)?;
    let _entry_selector = read_u16(data)?;
    let _range_shift = read_u16(data)?;

    let array_size = (segcount_x2 / 2) as usize;

    let mut end_code = Vec::with_capacity(array_size);
    let mut start_code = Vec::with_capacity(array_size);
    let mut id_delta = Vec::with_capacity(array_size);
    let mut id_range_offset = Vec::with_capacity(array_size);

    for i in 0..segcount_x2 / 2 {
        end_code.push(read_u16(data)?);
    }
    _ = read_u16(data)?; // Reserved padding, unused (who tf designed this, what were they going to put here???)
    for i in 0..segcount_x2 / 2 {
        start_code.push(read_u16(data)?);
    }
    for i in 0..segcount_x2 / 2 {
        id_delta.push(read_u16(data)?);
    }
    for i in 0..segcount_x2 / 2 {
        id_range_offset.push(read_u16(data)?);
    }

    let remaining_bytes = length - 12 - segcount_x2 * 4;
    let n_glyphs = remaining_bytes / 2; 
    let mut glyph_ids = Vec::with_capacity(n_glyphs as usize);

    for i in 0..n_glyphs {
        glyph_ids.push(read_u16(data)?);
    }

    Ok(Format4::new(language, start_code, end_code, id_delta, id_range_offset, glyph_ids))
}

pub struct Format4 {
    language: u16,
    start_code: Vec<u16>,
    end_code: Vec<u16>,
    id_delta: Vec<u16>,
    id_range_offset: Vec<u16>,
    glyph_ids: Vec<u16>,
}

impl Format4 {
    pub fn new(language: u16, start_code: Vec<u16>, end_code: Vec<u16>, id_delta: Vec<u16>, id_range_offset: Vec<u16>,
               glyph_ids: Vec<u16>) -> Self {
        assert_eq!(*end_code.last().unwrap(), 0xFFFF);
        Self {
            language: language,
            start_code: start_code,
            end_code: end_code,
            id_delta: id_delta,
            id_range_offset: id_range_offset,
            glyph_ids: glyph_ids,
        }
    }

    pub fn get_glyph_index(&self, codepoint: u16) -> Option<u16> {
        // TODO optimize. a lot.
        // Using some unsafe blocks here is probably fine, the spec
        // itself does some questionable pointer magic xD
        let mut index: usize = 0;
        while index < self.end_code.len() && self.end_code[index] < codepoint {
            index += 1;
        }

        if self.start_code[index] < codepoint {
            if self.id_range_offset[index] != 0 {
                let mut range_index = index + (self.id_range_offset[index]/2) as usize;
                range_index += (codepoint - self.start_code[index]) as usize;
                Some(self.glyph_ids[range_index % self.start_code.len()] + self.id_delta[index])
            } else {
                Some(codepoint + self.id_delta[index])
            }
        } else {
            None
        }

    }
}
