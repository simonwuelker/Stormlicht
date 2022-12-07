//! True Typeface parser
//! https://learn.microsoft.com/en-us/typography/opentype/spec/otff
//! https://formats.kaitai.io/ttf/index.html
//! https://handmade.network/forums/articles/t/7330-implementing_a_font_reader_and_rasterizer_from_scratch%252C_part_1__ttf_font_reader.

use crate::cmap;
use std::io::Cursor;
use std::io::Read;

#[derive(Debug)]
pub enum TTFParseError {
    UnexpectedEOF,
    UnsupportedFormat,
}

pub fn read_u32<R: Read>(bytes: &mut R) -> Result<u32, TTFParseError> {
    let mut u32_bytes = [0; 4];
    bytes
        .read_exact(&mut u32_bytes)
        .map_err(|_| TTFParseError::UnexpectedEOF)?;
    Ok(u32::from_be_bytes(u32_bytes))
}

pub fn read_u16<R: Read>(bytes: &mut R) -> Result<u16, TTFParseError> {
    let mut u16_bytes = [0; 2];
    bytes
        .read_exact(&mut u16_bytes)
        .map_err(|_| TTFParseError::UnexpectedEOF)?;
    Ok(u16::from_be_bytes(u16_bytes))
}

fn read_tag<R: Read>(bytes: &mut R) -> Result<[u8; 4], TTFParseError> {
    let mut tag_bytes = [0; 4];
    bytes
        .read_exact(&mut tag_bytes)
        .map_err(|_| TTFParseError::UnexpectedEOF)?;
    Ok(tag_bytes)
}

pub fn parse_font_face(bytes: &[u8]) -> Result<(), TTFParseError> {
    let mut cursor = Cursor::new(bytes);
    let sfnt_version = read_u32(&mut cursor)?;

    if sfnt_version != 0x00010000 {
        return Err(TTFParseError::UnsupportedFormat);
    }
    let num_tables = read_u16(&mut cursor)?;
    println!("{num_tables} tables");

    // These are legacy values which existed only for performance reasons.
    // We don't need them.
    let _search_range = read_u16(&mut cursor)?;
    let _entry_selector = read_u16(&mut cursor)?;
    let _range_shift = read_u16(&mut cursor)?;

    for _ in 0..num_tables {
        let tag = read_tag(&mut cursor)?;
        let checksum = read_u32(&mut cursor)?;
        let offset = read_u32(&mut cursor)?;
        let length = read_u32(&mut cursor)?;
        println!(
            "'{}' table length {}",
            String::from_utf8_lossy(&tag),
            length
        );

        match &tag {
            b"cmap" => {
                let old_position = cursor.position();
                cursor.set_position(offset as u64);
                let subtables = cmap::read_subtables(&mut cursor)?;
                for table in &subtables {
                    cursor.set_position((offset + table.offset) as u64);
                    let format = read_u16(&mut cursor)?;
                    if format == 4 {
                        let format_4 = cmap::read_format_4(&mut cursor)?;
                    }
                }
                cursor.set_position(old_position);
            },
            b"loca" => {
            },
            b"GDEF" => {
                // let _ = GDEFTable::read(&mut cursor);
            }
            _ => {},
        }
    }

    Ok(())
}
