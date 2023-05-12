#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color(pub u32);

impl Color {
    #[inline]
    pub const fn rgb(red: u8, green: u8, blue: u8) -> Self {
        Self((red as u32) << 24 | (green as u32) << 16 | (blue as u32) << 8)
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
