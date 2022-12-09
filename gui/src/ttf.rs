//! True Typeface parser
//! https://learn.microsoft.com/en-us/typography/opentype/spec/otff
//! https://formats.kaitai.io/ttf/index.html
//! https://handmade.network/forums/articles/t/7330-implementing_a_font_reader_and_rasterizer_from_scratch%252C_part_1__ttf_font_reader.
//!
//! Note that this module uses a significant amount of unsafe code.
//! That is by design, the `.ttf` specification by apple itself uses
//! a lot of questionable pointer magic.

use crate::tables::{cmap, head, loca};

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

#[derive(Debug)]
pub struct OffsetTable {
    scaler_type: u32,
    num_tables: u16,
    search_range: u16,
    entry_selector: u16,
    range_shift: u16,
}

pub trait Readable {
    fn read(data: &[u8]) -> Result<Self, TTFParseError>
    where
        Self: Sized;
}

impl Readable for OffsetTable {
    fn read(data: &[u8]) -> Result<Self, TTFParseError> {
        if data.len() < 12 {
            Err(TTFParseError::UnexpectedEOF)
        } else {
            Ok(Self {
                scaler_type: read_u32_at(data, 0),
                num_tables: read_u16_at(data, 4),
                search_range: read_u16_at(data, 6),
                entry_selector: read_u16_at(data, 8),
                range_shift: read_u16_at(data, 10),
            })
        }
    }
}

pub fn parse_font_face(bytes: &[u8]) -> Result<(), TTFParseError> {
    let offset_table = OffsetTable::read(&bytes)?;
    if offset_table.scaler_type != 0x00010000 {
        return Err(TTFParseError::UnsupportedFormat);
    }
    println!("{:?}", offset_table);

    // List tables
    for i in 0..offset_table.num_tables as usize{
        let ident = read_u32_at(bytes, 12 + 16 * i).to_be_bytes();
        println!("{}", std::str::from_utf8(&ident).unwrap());
    }

    let cmap_entry = get_table_entry(&bytes[12..], CMAP_TAG, offset_table.search_range as usize)
        .ok_or(TTFParseError::MissingTable)?;
    println!("{:?}", cmap_entry);

    let unicode_table_or_none = cmap::get_subtable_for_platform(
        &bytes[cmap_entry.offset as usize..],
        cmap::PlatformID::Unicode,
    );
    let unicode_table_offset = unicode_table_or_none.ok_or(TTFParseError::MissingTable)? as usize;
    let format4 =
        cmap::Format4::read(&bytes[cmap_entry.offset as usize + unicode_table_offset..]).unwrap();

    let head_entry = get_table_entry(&bytes[12..], HEAD_TAG, offset_table.search_range as usize)
        .ok_or(TTFParseError::MissingTable)?;
    let head_table = head::Head::read(&bytes[head_entry.offset as usize..]).unwrap();

    let loca_table = get_table_entry(&bytes[12..], LOCA_TAG, offset_table.search_range as usize).ok_or(TTFParseError::MissingTable)?;
    // let a_offset = loca::get_glyph_offset(&bytes[loca_table.offset as usize..], 622, head_table.index_to_loc_format)?;
    let glyf_table = get_table_entry(&bytes[2..], GLYF_TAG, offset_table.search_range as usize).ok_or(TTFParseError::MissingTable)?;


    println!("{:?}", head_table);

    Ok(())
}

#[derive(Debug)]
pub struct TableEntry {
    tag: u32,
    checksum: u32,
    offset: u32,
    length: u32,
}

impl Readable for TableEntry {
    fn read(data: &[u8]) -> Result<Self, TTFParseError> {
        if data.len() < 16 {
            Err(TTFParseError::UnexpectedEOF)
        } else {
            Ok(Self {
                tag: read_u32_at(data, 0),
                checksum: read_u32_at(data, 4),
                offset: read_u32_at(data, 8),
                length: read_u32_at(data, 12),
            })
        }
    }
}

/// Binary search for table type
/// TTF tables are always sorted alphabetically
fn get_table_entry(data: &[u8], target_ident: u32, search_range: usize) -> Option<TableEntry> {
    if data.len() < std::mem::size_of::<TableEntry>() {
        return None;
    }
    // make sure the index is always a multiple of the element size
    let index = (search_range / 2) & !0b1111 ;
    let ident = read_u32_at(&data, index);
    if ident == target_ident {
        Some(TableEntry::read(&data[index..]).unwrap())
    } else if ident < target_ident {
        get_table_entry(&data[index + 16..], target_ident, index)
    } else {
        get_table_entry(&data[..index], target_ident, index)
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
