use std::ops;

/// The base unit of measurement within CSS
///
/// Note that a CSS pixel is not necessarily equivalent to a
/// physical pixel on a screen. A CSS Pixel is always equal to `1/96in`.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct CSSPixels(pub f32);

impl CSSPixels {
    pub const ZERO: Self = Self(0.);
}

impl From<f32> for CSSPixels {
    fn from(value: f32) -> Self {
        Self(value)
    }
}

impl ops::Mul<f32> for CSSPixels {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self(self.0 * rhs)
    }
}
