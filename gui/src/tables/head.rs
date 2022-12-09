use crate::ttf::{read_i16_at, Readable, TTFParseError};


#[derive(Debug)]
pub struct Head {
    // version: Fixed,
    // font_revision: Fixed,
    // checksum_adjustment: u32,
    // magic: u32,
    // flags: u16,
    // units_per_em: u16,

    // // these are actually i64
    // created: u64,
    // modified: u64,

    // x_min: FWord,
    // y_min: FWord,
    // x_max: FWord,
    // y_max: FWord,

    // mac_style: u16,
    // lowest_rec_ppem: u16,
    // font_direction_hint: i16,
    pub index_to_loc_format: i16,
    // glyph_data_format: i16,
}

impl Readable for Head {
    fn read(data: &[u8]) -> Result<Self, TTFParseError> {
        if data.len() < 54 {
            return Err(TTFParseError::UnexpectedEOF);
        }
        let index_to_loc_format = read_i16_at(data, 50);

        Ok(Self {
            index_to_loc_format: index_to_loc_format,
        })
    }
}
