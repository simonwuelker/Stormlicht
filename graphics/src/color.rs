/// Pixel format (u32):
///
/// 00000000RRRRRRRRGGGGGGGGBBBBBBBB
///
/// 0: Bit is 0
/// R: Red channel
/// G: Green channel
/// B: Blue channel
///
/// This is the same format used by [softbuffer](https://docs.rs/softbuffer).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color(pub u32);

impl Color {
    pub const RED: Self = Self::rgb(255, 0, 0);
    pub const GREEN: Self = Self::rgb(0, 255, 0);
    pub const BLUE: Self = Self::rgb(0, 0, 255);
    pub const WHITE: Self = Self::rgb(255, 255, 255);
    pub const BLACK: Self = Self::rgb(0, 0, 0);

    #[inline]
    pub const fn rgb(red: u8, green: u8, blue: u8) -> Self {
        Self((red as u32) << 16 | (green as u32) << 8 | (blue as u32))
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::rgb(0, 0, 0)
    }
}

impl From<Color> for u32 {
    fn from(value: Color) -> Self {
        value.0
    }
}
