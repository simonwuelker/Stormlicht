#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec2D<T = f32> {
    pub x: T,
    pub y: T,
}

impl<T> Vec2D<T> {
    #[inline]
    pub const fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl Vec2D<f32> {
    #[inline]
    pub fn magnitude(&self) -> f32 {
        self.x.hypot(self.y)
    }

    #[inline]
    pub fn is_origin(&self) -> bool {
        self.magnitude() < f32::EPSILON
    }

    #[inline]
    pub fn angle(&self) -> Angle {
        Angle::from_radians(self.y.atan2(self.x))
    }

    #[inline]
    pub fn lerp(&self, other: Self, t: f32) -> Self {
        debug_assert!(0. <= t);
        debug_assert!(t <= 1.);

        Self {
            x: (other.x - self.x).mul_add(t, self.x),
            y: (other.y - self.y).mul_add(t, self.y),
        }
    }

    // Compute the dot product of two vectors
    #[inline]
    pub fn dot(&self, other: Self) -> f32 {
        self.x.mul_add(other.x, self.y * other.y)
    }

    // Compute the cross product of two vectors
    #[inline]
    pub fn cross_product(&self, other: Self) -> f32 {
        self.x.mul_add(other.y, -self.y * other.x)
    }

    #[inline]
    pub fn round_to_grid(&self) -> Vec2D<usize> {
        Vec2D {
            x: self.x.round() as usize,
            y: self.y.round() as usize,
        }
    }
}

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
    pub fn from_radians(radians: f32) -> Self {
        Self(radians)
    }

    #[inline]
    pub fn diff(&self, other: &Self) -> Self {
        let mut difference_in_radians = (self.0 - other.0).abs();

        if std::f32::consts::PI < difference_in_radians {
            difference_in_radians = std::f32::consts::TAU - difference_in_radians;
        }

        Self(difference_in_radians)
    }

    #[inline]
    pub fn sin(&self) -> f32 {
        self.0.sin()
    }

    #[inline]
    pub fn cos(&self) -> f32 {
        self.0.cos()
    }
}

impl PartialEq for Angle {
    fn eq(&self, other: &Self) -> bool {
        self.diff(other).0 < Self::MAX_ERROR
    }
}

impl std::ops::Add for Vec2D {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl std::ops::Sub for Vec2D {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl std::ops::Mul<f32> for Vec2D {
    type Output = Vec2D;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Angle, Vec2D};

    #[test]
    fn magnitude() {
        let vec = Vec2D::new(1., 1.);
        assert_eq!(vec.magnitude(), std::f32::consts::SQRT_2);
    }

    #[test]
    fn compute_angle() {
        assert_eq!(Vec2D::new(1., 0.).angle(), Angle::from_radians(0.));
        assert_eq!(
            Vec2D::new(-1., 1.).angle(),
            Angle::from_radians(3. * std::f32::consts::FRAC_PI_4)
        );
        assert_eq!(
            Vec2D::new(0., -1.).angle(),
            Angle::from_radians(-std::f32::consts::FRAC_PI_2)
        );
    }
}
