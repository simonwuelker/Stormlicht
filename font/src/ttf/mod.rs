//! [TrueType](https://developer.apple.com/fonts/TrueType-Reference-Manual) font parser
//!
//! ## Reference Material:
//! * <https://learn.microsoft.com/en-us/typography/opentype/spec/otff>
//! * <https://formats.kaitai.io/ttf/index.html>
//! * <https://handmade.network/forums/articles/t/7330-implementing_a_font_reader_and_rasterizer_from_scratch%252C_part_1__ttf_font_reader>

pub mod tables;
use tables::{cmap, glyf, head, hhea, hmtx, loca, maxp, offset::OffsetTable};

use self::tables::name;

const CMAP_TAG: u32 = u32::from_be_bytes(*b"cmap");
const HEAD_TAG: u32 = u32::from_be_bytes(*b"head");
const LOCA_TAG: u32 = u32::from_be_bytes(*b"loca");
const GLYF_TAG: u32 = u32::from_be_bytes(*b"glyf");
const HHEA_TAG: u32 = u32::from_be_bytes(*b"hhea");
const HMTX_TAG: u32 = u32::from_be_bytes(*b"hmtx");
const MAXP_TAG: u32 = u32::from_be_bytes(*b"maxp");
const NAME_TAG: u32 = u32::from_be_bytes(*b"name");

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
    glyph_table: glyf::GlyphOutlineTable<'a>,
    _hmtx_table: hmtx::HMTXTable<'a>,
    maxp_table: maxp::MaxPTable<'a>,
    name_table: name::NameTable<'a>,
}

impl<'a> Font<'a> {
    pub fn new(data: &'a [u8]) -> Result<Self, TTFParseError> {
        let offset_table = OffsetTable::new(data);
        if offset_table.scaler_type() != 0x00010000 {
            return Err(TTFParseError::UnsupportedFormat);
        }

        let head_entry = offset_table
            .get_table(HEAD_TAG)
            .ok_or(TTFParseError::MissingTable)?;
        let head_table = head::HeadTable::new(data, head_entry.offset());

        let cmap_entry = offset_table
            .get_table(CMAP_TAG)
            .ok_or(TTFParseError::MissingTable)?;
        let cmap_table = cmap::CMAPTable::new(data, cmap_entry.offset());

        let unicode_table_or_none = cmap_table.get_subtable_for_platform(cmap::PlatformID::Unicode);

        let unicode_table_offset = unicode_table_or_none.ok_or(TTFParseError::MissingTable)?;
        let format4 = cmap::Format4::new(data, cmap_entry.offset() + unicode_table_offset);

        let loca_entry = offset_table
            .get_table(LOCA_TAG)
            .ok_or(TTFParseError::MissingTable)?;
        let loca_table =
            loca::LocaTable::new(data, loca_entry.offset(), head_table.index_to_loc_format());

        let glyf_entry = offset_table
            .get_table(GLYF_TAG)
            .ok_or(TTFParseError::MissingTable)?;
        let glyph_table = glyf::GlyphOutlineTable::new(
            data,
            glyf_entry.offset(),
            glyf_entry.length(),
            loca_table,
        );

        let hhea_entry = offset_table
            .get_table(HHEA_TAG)
            .ok_or(TTFParseError::MissingTable)?;
        let hhea_table = hhea::HHEATable::new(data, hhea_entry.offset());

        let hmtx_entry = offset_table
            .get_table(HMTX_TAG)
            .ok_or(TTFParseError::MissingTable)?;
        let hmtx_table = hmtx::HMTXTable::new(
            data,
            hmtx_entry.offset(),
            hhea_table.num_of_long_hor_metrics(),
        );

        let maxp_entry = offset_table
            .get_table(MAXP_TAG)
            .ok_or(TTFParseError::MissingTable)?;
        let maxp_table = maxp::MaxPTable::new(data, maxp_entry.offset());

        let name_entry = offset_table
            .get_table(NAME_TAG)
            .ok_or(TTFParseError::MissingTable)?;
        let name_table = name::NameTable::new(data, name_entry.offset()).unwrap();

        Ok(Self {
            offset_table: offset_table,
            head_table: head_table,
            format4: format4,
            glyph_table: glyph_table,
            _hmtx_table: hmtx_table,
            maxp_table: maxp_table,
            name_table: name_table,
        })
    }

    pub fn num_glyphs(&self) -> usize {
        self.maxp_table.num_glyphs()
    }

    // TODO: support more than one cmap format table (format 4 seems to be the most common but still)
    pub fn format_4(&self) -> &cmap::Format4<'a> {
        &self.format4
    }

    pub fn name(&self) -> &name::NameTable<'a> {
        &self.name_table
    }

    pub fn glyf(&self) -> &glyf::GlyphOutlineTable<'a> {
        &self.glyph_table
    }

    pub fn offset_table(&self) -> &OffsetTable<'a> {
        &self.offset_table
    }

    pub fn get_glyph(&self, codepoint: u16) -> Result<glyf::Glyph<'a>, TTFParseError> {
        // Any character that does not exist is mapped to index zero, which is defined to be the
        // missing character glyph
        let glyph_index = self.format4.get_glyph_index(codepoint).unwrap_or(0);
        Ok(self.glyph_table.get_glyph(glyph_index))
    }

    /// Return the number of coordinate points per font size unit.
    /// This value is used to scale fonts, ie. when you render a font with
    /// size `17px`, one `em` equals `17px`.
    ///
    /// Note that this value does not constrain the size of individual glyphs.
    /// A glyph may have a size larger than `1em`.
    pub fn units_per_em(&self) -> u16 {
        self.head_table.units_per_em()
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
