//! True Typeface parser
//! https://learn.microsoft.com/en-us/typography/opentype/spec/otff
//! https://formats.kaitai.io/ttf/index.html
//! https://handmade.network/forums/articles/t/7330-implementing_a_font_reader_and_rasterizer_from_scratch%252C_part_1__ttf_font_reader.

use crate::tables::{cmap, glyf, head, loca, offset};

const CMAP_TAG: u32 = u32::from_be_bytes(*b"cmap");
const HEAD_TAG: u32 = u32::from_be_bytes(*b"head");
const LOCA_TAG: u32 = u32::from_be_bytes(*b"loca");
const GLYF_TAG: u32 = u32::from_be_bytes(*b"glyf");

#[derive(Debug)]
pub enum TTFParseError {
    UnexpectedEOF,
    UnsupportedFormat,
    MissingTable,
}

pub struct Font<'a> {
    data: &'a [u8],
    offset_table: offset::OffsetTable<'a>,
    head_table: head::HeadTable<'a>,
    format4: cmap::Format4<'a>,
    loca_table: loca::LocaTable<'a>,
    glyph_table: glyf::GlyphOutlineTable<'a>,
}

impl<'a> Font<'a> {
    pub fn new(data: &'a [u8]) -> Result<Self, TTFParseError> {
        let offset_table = offset::OffsetTable::new(&data);
        if offset_table.scaler_type() != 0x00010000 {
            return Err(TTFParseError::UnsupportedFormat);
        }

        let head_entry = offset_table
            .get_table(HEAD_TAG)
            .ok_or(TTFParseError::MissingTable)?;
        let head_table = head::HeadTable::new(&data, head_entry.offset());

        let cmap_entry = offset_table
            .get_table(CMAP_TAG)
            .ok_or(TTFParseError::MissingTable)?;
        let cmap_table = cmap::CMAPTable::new(&data, cmap_entry.offset());

        let unicode_table_or_none = cmap_table.get_subtable_for_platform(cmap::PlatformID::Unicode);

        let unicode_table_offset = unicode_table_or_none.ok_or(TTFParseError::MissingTable)?;
        let format4 = cmap::Format4::new(&data, cmap_entry.offset() + unicode_table_offset);

        let loca_entry = offset_table
            .get_table(LOCA_TAG)
            .ok_or(TTFParseError::MissingTable)?;
        let loca_table = loca::LocaTable::new(&data, loca_entry.offset());

        let glyf_entry = offset_table
            .get_table(GLYF_TAG)
            .ok_or(TTFParseError::MissingTable)?;
        let glyph_table =
            glyf::GlyphOutlineTable::new(&data, glyf_entry.offset(), glyf_entry.length());

        Ok(Self {
            data: data,
            offset_table: offset_table,
            head_table: head_table,
            format4: format4,
            loca_table: loca_table,
            glyph_table: glyph_table,
        })
    }

    pub fn get_glyph(&self, codepoint: u16) -> Result<glyf::GlyphOutline<'a>, TTFParseError> {
        // Any character that does not exist is mapped to index zero, which is defined to be the
        // missing character glyph
        let glyph_index = self.format4.get_glyph_index(codepoint).unwrap_or(0);

        let glyph_offset = self
            .loca_table
            .get_glyph_offset(glyph_index, self.head_table.index_to_loc_format())?;
        Ok(self.glyph_table.get_glyph_outline(glyph_offset))
    }
}

pub fn read_u16_at(data: &[u8], offset: usize) -> u16 {
    u16::from_be_bytes(data[offset..offset + 2].try_into().unwrap())
}

pub fn read_u32_at(data: &[u8], offset: usize) -> u32 {
    u32::from_be_bytes(data[offset..offset + 4].try_into().unwrap())
}

pub fn read_i16_at(data: &[u8], offset: usize) -> i16 {
    i16::from_be_bytes(data[offset..offset + 2].try_into().unwrap())
}
