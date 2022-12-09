//! True Typeface parser
//! https://learn.microsoft.com/en-us/typography/opentype/spec/otff
//! https://formats.kaitai.io/ttf/index.html
//! https://handmade.network/forums/articles/t/7330-implementing_a_font_reader_and_rasterizer_from_scratch%252C_part_1__ttf_font_reader.

use crate::tables::{cmap, head, loca, offset, glyf};

const CMAP_TAG: u32 = u32::from_be_bytes(*b"cmap");
const HEAD_TAG: u32 = u32::from_be_bytes(*b"head");
const LOCA_TAG: u32 = u32::from_be_bytes(*b"loca");
const GLYF_TAG: u32 = u32::from_be_bytes(*b"glyf");

pub type Fixed = u32;
pub type FWord = i16;

#[derive(Debug)]
pub enum TTFParseError {
    UnexpectedEOF,
    UnsupportedFormat,
    MissingTable,
}

pub trait Readable {
    fn read(data: &[u8]) -> Result<Self, TTFParseError>
    where
        Self: Sized;
}

pub fn parse_font_face(bytes: &[u8]) -> Result<(), TTFParseError> {
    let offset_table = offset::OffsetTable::new(&bytes);
    if offset_table.scaler_type() != 0x00010000 {
        return Err(TTFParseError::UnsupportedFormat);
    }
    println!("{:?}", offset_table);

    // List tables
    for i in 0..offset_table.num_tables() {
        let ident = read_u32_at(bytes, 12 + 16 * i).to_be_bytes();
        println!("{}", std::str::from_utf8(&ident).unwrap());
    }

    let cmap_entry = offset_table.get_table(CMAP_TAG)
        .ok_or(TTFParseError::MissingTable)?;
    println!("{:?}", cmap_entry);

    let cmap_table = cmap::CMAPTable::new(&bytes, cmap_entry.offset());
    let unicode_table_or_none = cmap_table.get_subtable_for_platform(cmap::PlatformID::Unicode);

    let unicode_table_offset = unicode_table_or_none.ok_or(TTFParseError::MissingTable)?;
    let format4 = cmap::Format4::new(&bytes, cmap_entry.offset() + unicode_table_offset);

    let head_entry = offset_table.get_table(HEAD_TAG)
        .ok_or(TTFParseError::MissingTable)?;
    let head_table = head::Head::new(&bytes, head_entry.offset());

    let loca_table = offset_table.get_table(LOCA_TAG).ok_or(TTFParseError::MissingTable)?;
    let a_offset = loca::get_glyph_offset(&bytes[loca_table.offset()..], 622, head_table.index_to_loc_format())?;

    println!("A => {:?}", format4.get_glyph_index(65));
    println!("A is {a_offset}");

    let glyf_entry = offset_table.get_table(GLYF_TAG).ok_or(TTFParseError::MissingTable).unwrap();
    println!("{:?}", glyf_entry);
    let glyph_table = glyf::GlyphOutlineTable::new(&bytes, glyf_entry.offset(), glyf_entry.length());
    let a_glyph = glyph_table.get_glyph_outline(a_offset);


    println!("{:?}", a_glyph);

    Ok(())
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
