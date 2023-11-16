use std::ops;

/// The base unit of measurement within CSS
///
/// Note that a CSS pixel is not necessarily equivalent to a
/// physical pixel on a screen. A CSS Pixel is always equal to `1/96in`.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Pixels(pub f32);

impl Pixels {
    pub const ZERO: Self = Self(0.);
}

impl From<f32> for Pixels {
    fn from(value: f32) -> Self {
        Self(value)
    }
}

impl From<Pixels> for f32 {
    fn from(value: Pixels) -> Self {
        value.0
    }
}

impl ops::Mul<f32> for Pixels {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl ops::Div<f32> for Pixels {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self(self.0 / rhs)
    }
}

impl ops::Add for Pixels {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl ops::AddAssign for Pixels {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0
    }
}

impl ops::Sub for Pixels {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl ops::SubAssign for Pixels {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0
    }
}

impl Default for Pixels {
    fn default() -> Self {
        Self::ZERO
    }
}
