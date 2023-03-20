//! [Name](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6name.html) table implementation.
//!
//! Note that this implementation includes *some* features from the [OpenType Name Table](https://learn.microsoft.com/en-us/typography/opentype/spec/name)

use crate::ttf::read_u16_at;
use anyhow::Result;
use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NameID {
    Copyright,
    FontFamily,
    FontSubFamily,
    UniqueSubFamilyIdentification,
    FullName,
    NameTableVersion,
    PostScriptName,
    Trademark,
    Manufacturer,
    Designer,
    Description,
    VendorURL,
    DesignerURL,
    License,
    LicenseURL,
    Reserved,
    PreferredFamily,
    PreferredSubFamily,
    CompatibleFull,
    SampleText,
    PostScriptCIDFindFontName,
    WWSFamilyName,
    WWSSubFamilyName,
    LightBackgroundPalette,
    DarkBackgroundPalette,
    VariationsPostscriptNamePrefix,
    FontSpecific,
}

impl From<u16> for NameID {
    fn from(value: u16) -> Self {
        match value {
            0 => Self::Copyright,
            1 => Self::FontFamily,
            2 => Self::FontSubFamily,
            3 => Self::UniqueSubFamilyIdentification,
            4 => Self::FullName,
            5 => Self::NameTableVersion,
            6 => Self::PostScriptName,
            7 => Self::Trademark,
            8 => Self::Manufacturer,
            9 => Self::Designer,
            10 => Self::Description,
            11 => Self::VendorURL,
            12 => Self::DesignerURL,
            13 => Self::License,
            14 => Self::LicenseURL,
            15 => Self::Reserved,
            16 => Self::PreferredFamily,
            17 => Self::PreferredSubFamily,
            18 => Self::CompatibleFull,
            19 => Self::SampleText,
            20 => Self::PostScriptCIDFindFontName,
            21 => Self::WWSFamilyName,
            22 => Self::WWSSubFamilyName,
            23 => Self::LightBackgroundPalette,
            24 => Self::DarkBackgroundPalette,
            25 => Self::VariationsPostscriptNamePrefix,
            26..=255 => Self::Reserved,
            256.. => Self::FontSpecific,
        }
    }
}

#[derive(Debug, Error)]
pub enum NameTableError {
    #[error("Expected format selector 0, found {}", .0)]
    NonZeroFormatSelector(u16),
}
pub struct NameTable<'a>(&'a [u8]);

impl<'a> NameTable<'a> {
    pub fn new(data: &'a [u8], offset: usize) -> Result<Self> {
        let format_selector = read_u16_at(data, offset);
        if format_selector != 0 {
            return Err(NameTableError::NonZeroFormatSelector(format_selector).into());
        }
        Ok(Self(&data[offset..]))
    }

    /// Get the number of name records in this table
    pub fn num_records(&self) -> u16 {
        read_u16_at(self.0, 2)
    }

    /// Get the full name of the font, if any.
    pub fn get_font_name(&self) -> Option<String> {
        self.name_records()
            .filter(|name_record| name_record.name_id == NameID::FullName)
            .map(|name_record| name_record.value)
            .nth(0)
    }

    /// Get an iterator over the suitable name records from the font.
    ///
    /// Only Unicode name records are considered "suitable".
    pub fn name_records(&self) -> NameRecordIterator {
        NameRecordIterator {
            data: self.0,
            index: 0,
            num_records: self.num_records() as usize,
            string_offset: read_u16_at(self.0, 4) as usize,
        }
    }
}

#[derive(Clone, Debug)]
pub struct NameRecord {
    pub name_id: NameID,
    pub value: String,
}

pub struct NameRecordIterator<'a> {
    data: &'a [u8],
    index: usize,
    num_records: usize,
    string_offset: usize,
}

impl<'a> Iterator for NameRecordIterator<'a> {
    type Item = NameRecord;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.index == self.num_records {
                return None;
            }

            self.index += 1;

            let base = 6 + self.index * 12;
            let platform_id = read_u16_at(self.data, base);
            let platform_specific_id = read_u16_at(self.data, base + 2);

            // We only support unicode encoding
            // From my understanding, everything else is pretty much
            // deprecated anyways
            // Unicode is either platform ID 0 (unicode) or platform id 3 (microsoft) with specific id 1 (also unicode)
            if platform_id != 0 && !(platform_id == 3 && platform_specific_id == 1) {
                continue;
            }

            let name_id = read_u16_at(self.data, base + 6).into();
            let length = read_u16_at(self.data, base + 8) as usize;
            let offset = self.string_offset + read_u16_at(self.data, base + 10) as usize;

            let value_bytes = &self.data[offset..][..length];

            // The bytes are in big-endian order, we need to convert to native endianness
            let mut native_u16s = Vec::with_capacity(value_bytes.len() / 2);
            for i in (0..value_bytes.len()).step_by(2) {
                native_u16s.push(u16::from_be_bytes(
                    value_bytes[i..i + 2].try_into().unwrap(),
                ));
            }

            let value = String::from_utf16_lossy(&native_u16s);
            return Some(NameRecord {
                name_id: name_id,
                value: value,
            });
        }
    }
}
