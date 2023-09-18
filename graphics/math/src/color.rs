use std::fmt;

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
#[derive(Clone, Copy, PartialEq)]
pub struct Color(pub u32);

impl Color {
    pub const RED: Self = Self::rgb(255, 0, 0);
    pub const GREEN: Self = Self::rgb(0, 255, 0);
    pub const BLUE: Self = Self::rgb(0, 0, 255);
    pub const WHITE: Self = Self::rgb(255, 255, 255);
    pub const BLACK: Self = Self::rgb(0, 0, 0);

    #[inline]
    #[must_use]
    pub const fn rgb(red: u8, green: u8, blue: u8) -> Self {
        Self((red as u32) << 16 | (green as u32) << 8 | (blue as u32))
    }

    #[inline]
    #[must_use]
    pub const fn inverted(&self) -> Self {
        Self::rgb(
            u8::MAX - self.red(),
            u8::MAX - self.green(),
            u8::MAX - self.blue(),
        )
    }

    #[inline]
    #[must_use]
    pub const fn red(&self) -> u8 {
        (self.0 >> 16) as u8
    }

    #[inline]
    #[must_use]
    pub const fn green(&self) -> u8 {
        (self.0 >> 8) as u8
    }

    #[inline]
    #[must_use]
    pub const fn blue(&self) -> u8 {
        self.0 as u8
    }

    pub fn interpolate(&self, other: Self, opacity: f32) -> Self {
        if opacity == 1. {
            *self
        } else if opacity == 0. {
            other
        } else {
            Self::rgb(
                (self.red() as f32 * opacity + other.red() as f32 * (1. - opacity)).round() as u8,
                (self.green() as f32 * opacity + other.green() as f32 * (1. - opacity)).round()
                    as u8,
                (self.blue() as f32 * opacity + other.blue() as f32 * (1. - opacity)).round() as u8,
            )
        }
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
impl fmt::Debug for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "rbg({}, {}, {})", self.red(), self.green(), self.blue())
    }
}
