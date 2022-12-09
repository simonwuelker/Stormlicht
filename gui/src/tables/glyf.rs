use crate::ttf::{read_i16_at, read_u16_at, Readable, TTFParseError};

#[derive(Debug)]
pub struct Glyf {
    num_contours: u16,
    x_min: i16,
    y_min: i16,
    x_max: i16,
    y_max: i16,
    instruction_length: u16,
    instructions: Vec<u8>,
    flags: Vec<GlyfFlag>,
    x_coordinates: Vec<i16>,
    y_coordinates: Vec<i16>,
    end_points: Vec<u16>,
}

impl Readable for Glyf {
    fn read(data: &[u8]) -> Result<Self, TTFParseError> {
        if data.len() < 12 {
            return Err(TTFParseError::UnexpectedEOF);
        }

        let num_contours = read_u16_at(data, 0);
        let x_min = read_i16_at(data, 2);
        let y_min = read_i16_at(data, 4);
        let x_max = read_i16_at(data, 6);
        let y_max = read_i16_at(data, 8);
        let instruction_length = read_u16_at(data, 10);
        todo!()

    }
}

#[derive(Debug)]
pub struct GlyfFlag(u8);

impl GlyfFlag {
    const POINT_ON_CURVE: u8 = 1;
    const SHORT_X: u8 = 2;
    const SHORT_Y: u8 = 4;
    const REPEAT: u8 = 8;

    pub fn is_on_curve(&self) -> bool {
        self.0 & Self::POINT_ON_CURVE != 0
    }

    pub fn is_short_x(&self) -> bool {
        self.0 & Self::SHORT_X != 0
    }

    pub fn is_short_y(&self) -> bool {
        self.0 & Self::SHORT_Y != 0
    }

    pub fn repeat(&self) -> bool {
        self.0 & Self::REPEAT != 0
    }
}
