#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    #[inline]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    #[inline]
    pub fn magnitude(&self) -> f32 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }

    #[inline]
    pub fn is_origin(&self) -> bool {
        self.magnitude() < f32::EPSILON
    }

    #[inline]
    pub fn angle(&self) -> Angle {
        Angle::from_radians(self.y.atan2(self.x))
    }
}

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
}

impl PartialEq for Angle {
    fn eq(&self, other: &Self) -> bool {
        self.diff(other).0 < Self::MAX_ERROR
    }
}

impl std::ops::Add for Point {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl std::ops::Sub for Point {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Angle, Point};

    #[test]
    fn magnitude() {
        let point = Point::new(1., 1.);
        assert_eq!(point.magnitude(), std::f32::consts::SQRT_2);
    }

    #[test]
    fn compute_angle() {
        assert_eq!(Point::new(1., 0.).angle(), Angle::from_radians(0.));
        assert_eq!(
            Point::new(-1., 1.).angle(),
            Angle::from_radians(3. * std::f32::consts::FRAC_PI_4)
        );
        assert_eq!(
            Point::new(0., -1.).angle(),
            Angle::from_radians(-std::f32::consts::FRAC_PI_2)
        );
    }
}
