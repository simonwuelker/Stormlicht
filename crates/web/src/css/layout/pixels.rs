use std::ops;

/// The base unit of measurement within CSS
///
/// Note that a CSS pixel is not necessarily equivalent to a
/// physical pixel on a screen. A CSS Pixel is always equal to `1/96in`.

#[derive(Clone, Copy, Debug)]
pub struct Pixels(pub f32);

impl Pixels {
    pub const ZERO: Self = Self(0.);
    pub const INFINITY: Self = Self(f32::INFINITY);

    /// Returns `true` if `self.0` has a negative sign, including `-0.0`
    #[inline]
    #[must_use]
    pub fn is_sign_negative(&self) -> bool {
        self.0.is_sign_negative()
    }

    /// Return the numerical value in pixels
    #[inline]
    #[must_use]
    pub const fn value(&self) -> f32 {
        self.0
    }
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

impl ops::Mul<Pixels> for f32 {
    type Output = Pixels;

    fn mul(self, rhs: Pixels) -> Self::Output {
        Pixels(self * rhs.0)
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

impl PartialOrd for Pixels {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Pixels {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.total_cmp(&other.0)
    }
}

impl PartialEq for Pixels {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

impl Eq for Pixels {}
