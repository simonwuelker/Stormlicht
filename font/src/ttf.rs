//! True Typeface parser
//! https://learn.microsoft.com/en-us/typography/opentype/spec/otff
//! https://formats.kaitai.io/ttf/index.html
//! https://handmade.network/forums/articles/t/7330-implementing_a_font_reader_and_rasterizer_from_scratch%252C_part_1__ttf_font_reader.

use crate::tables::{cmap, glyf, head, loca, offset::{OffsetTable}, hhea, hmtx};

const CMAP_TAG: u32 = u32::from_be_bytes(*b"cmap");
const HEAD_TAG: u32 = u32::from_be_bytes(*b"head");
const LOCA_TAG: u32 = u32::from_be_bytes(*b"loca");
const GLYF_TAG: u32 = u32::from_be_bytes(*b"glyf");
const HHEA_TAG: u32 = u32::from_be_bytes(*b"hhea");
const HMTX_TAG: u32 = u32::from_be_bytes(*b"hmtx");

#[derive(Debug)]
pub enum TTFParseError {
    UnexpectedEOF,
    UnsupportedFormat,
    MissingTable,
}

pub struct Font<'a> {
    offset_table: OffsetTable<'a>,
    head_table: head::HeadTable<'a>,
    format4: cmap::Format4<'a>,
    loca_table: loca::LocaTable<'a>,
    glyph_table: glyf::GlyphOutlineTable<'a>,
    hmtx_table: hmtx::HMTXTable<'a>,
}

const DEFAULT_FONT: &'static [u8; 168644] = include_bytes!("../../downloads/fonts/roboto/Roboto-Medium.ttf");

impl<'a> Font<'a> {
    pub fn new(data: &'a [u8]) -> Result<Self, TTFParseError> {
        let offset_table = OffsetTable::new(&data);
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

        let hhea_entry = offset_table
            .get_table(HHEA_TAG)
            .ok_or(TTFParseError::MissingTable)?;
        let hhea_table =
            hhea::HHEATable::new(&data, hhea_entry.offset());

        let hmtx_entry = offset_table
            .get_table(HMTX_TAG)
            .ok_or(TTFParseError::MissingTable)?;
        let hmtx_table =
            hmtx::HMTXTable::new(&data, hmtx_entry.offset(), hhea_table.num_of_long_hor_metrics());

        Ok(Self {
            offset_table: offset_table,
            head_table: head_table,
            format4: format4,
            loca_table: loca_table,
            glyph_table: glyph_table,
            hmtx_table: hmtx_table,
        })
    }

    pub fn get_glyph(&self, codepoint: u16) -> Result<glyf::Glyph<'a>, TTFParseError> {
        // Any character that does not exist is mapped to index zero, which is defined to be the
        // missing character glyph
        let glyph_index = self.format4.get_glyph_index(codepoint).unwrap_or(0);

        let glyph_offset = self
            .loca_table
            .get_glyph_offset(glyph_index, self.head_table.index_to_loc_format())?;
        Ok(self.glyph_table.get_glyph(glyph_offset))
    }

    /// Compute the rendered width of a given character sequence
    pub fn compute_width(&self, text: &str) -> usize {
        let mut total_length = 0;

        for c in text.chars() {
            let glyph_index = self.format4.get_glyph_index(c as u16).unwrap_or(0);
            total_length += self.hmtx_table.get_metric_for(glyph_index).advance_width() as usize;
        }
        total_length
    }

    pub fn offset_table(&self) -> &OffsetTable<'a> {
        &self.offset_table
    }
}

impl Default for Font<'static> {
    fn default() -> Self {
        Self::new(DEFAULT_FONT).unwrap()
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
