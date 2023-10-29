/// Zero cost wrapper type for an `f32`.
///
/// This type exists since coordinates are also `f32`'s.
/// It should enforce type safety to prevent coordinates from accidentally being
/// used as angles.
#[derive(Clone, Copy, Debug, Default)]
pub struct Angle(f32);

impl Angle {
    /// Angles with a difference below this value (in radians) are considered equal
    const MAX_ERROR: f32 = 0.01;

    #[inline]
    #[must_use]
    pub fn from_radians(radians: f32) -> Self {
        Self(radians)
    }

    #[inline]
    #[must_use]
    pub fn diff(&self, other: &Self) -> Self {
        let mut difference_in_radians = (self.0 - other.0).abs();

        if std::f32::consts::PI < difference_in_radians {
            difference_in_radians = std::f32::consts::TAU - difference_in_radians;
        }

        Self(difference_in_radians)
    }

    #[inline]
    #[must_use]
    pub fn sin(&self) -> f32 {
        self.0.sin()
    }

    #[inline]
    #[must_use]
    pub fn cos(&self) -> f32 {
        self.0.cos()
    }
}

impl PartialEq for Angle {
    #[must_use]
    fn eq(&self, other: &Self) -> bool {
        self.diff(other).0 < Self::MAX_ERROR
    }
}
