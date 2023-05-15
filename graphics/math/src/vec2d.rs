use crate::Angle;
use std::ops::{Add, Div, Mul, Sub};

/// Generate a trait impl for an operation involving two [Vec2D]s, like [Add] or [Sub]
macro_rules! impl_bin_op {
    ($trait: ident, $fn: ident, $op: tt) => {
        impl<T: $trait<T, Output = T>> $trait for Vec2D<T> {
            type Output = Vec2D<T>;

            #[must_use]
            fn $fn(self, rhs: Self) -> Self::Output {
                Self {
                    x: self.x $op rhs.x,
                    y: self.y $op rhs.y,
                }
            }
        }
    };
}

/// Generate a trait impl for an operation involving a [Vec2D] and a scalar value of unknown type
macro_rules! impl_scalar_op {
    ($trait: ident, $fn: ident, $op: tt, $rhs: ident) => {
        impl<T: $trait<$rhs, Output = T>> $trait<$rhs> for Vec2D<T> {
            type Output = Vec2D<T>;

            #[must_use]
            fn $fn(self, rhs: $rhs) -> Self::Output {
                Self {
                    x: self.x $op rhs,
                    y: self.y $op rhs,
                }
            }
        }
    };
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec2D<T = f32> {
    pub x: T,
    pub y: T,
}

impl<T> Vec2D<T> {
    #[inline]
    #[must_use]
    pub const fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl<T: Add<Output = T> + Div<i32, Output = T>> Vec2D<T> {
    #[inline]
    #[must_use]
    pub fn middle(a: Self, b: Self) -> Self {
        Self::new((a.x + b.x) / 2, (a.y + b.y) / 2)
    }
}

impl Vec2D<f32> {
    #[inline]
    #[must_use]
    pub fn magnitude(&self) -> f32 {
        self.x.hypot(self.y)
    }

    #[inline]
    #[must_use]
    pub fn is_origin(&self) -> bool {
        self.magnitude() < f32::EPSILON
    }

    #[inline]
    #[must_use]
    pub fn angle(&self) -> Angle {
        Angle::from_radians(self.y.atan2(self.x))
    }

    #[inline]
    #[must_use]
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
    #[must_use]
    pub fn dot(&self, other: Self) -> f32 {
        self.x.mul_add(other.x, self.y * other.y)
    }

    // Compute the cross product of two vectors
    #[inline]
    #[must_use]
    pub fn cross_product(&self, other: Self) -> f32 {
        self.x.mul_add(other.y, -self.y * other.x)
    }

    #[inline]
    #[must_use]
    pub fn round_to_grid(&self) -> Vec2D<usize> {
        Vec2D {
            x: self.x.round() as usize,
            y: self.y.round() as usize,
        }
    }
}

impl_bin_op!(Add, add, +);
impl_bin_op!(Sub, sub, -);

impl_scalar_op!(Mul, mul, *, f32);
impl_scalar_op!(Mul, mul, *, f64);
impl_scalar_op!(Mul, mul, *, i8);
impl_scalar_op!(Mul, mul, *, i16);
impl_scalar_op!(Mul, mul, *, i32);
impl_scalar_op!(Mul, mul, *, i64);
impl_scalar_op!(Mul, mul, *, i128);
impl_scalar_op!(Mul, mul, *, u8);
impl_scalar_op!(Mul, mul, *, u16);
impl_scalar_op!(Mul, mul, *, u32);
impl_scalar_op!(Mul, mul, *, u64);
impl_scalar_op!(Mul, mul, *, u128);

impl_scalar_op!(Div, div, /, f32);
impl_scalar_op!(Div, div, /, f64);
impl_scalar_op!(Div, div, /, i8);
impl_scalar_op!(Div, div, /, i16);
impl_scalar_op!(Div, div, /, i32);
impl_scalar_op!(Div, div, /, i64);
impl_scalar_op!(Div, div, /, i128);
impl_scalar_op!(Div, div, /, u8);
impl_scalar_op!(Div, div, /, u16);
impl_scalar_op!(Div, div, /, u32);
impl_scalar_op!(Div, div, /, u64);
impl_scalar_op!(Div, div, /, u128);

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

    #[test]
    fn vec2d_add() {
        assert_eq!(Vec2D::new(1., 0.) + Vec2D::new(1., 1.), Vec2D::new(2., 1.));
        assert_eq!(Vec2D::new(-2, 1) + Vec2D::new(1, 0), Vec2D::new(-1, 1));
    }

    #[test]
    fn vec2d_sub() {
        assert_eq!(Vec2D::new(1., 0.) - Vec2D::new(1., 1.), Vec2D::new(0., -1.));
        assert_eq!(Vec2D::new(-2, 1) - Vec2D::new(1, 0), Vec2D::new(-3, 1));
    }

    #[test]
    fn vec2d_mul() {
        assert_eq!(Vec2D::new(1., -1.) * 2., Vec2D::new(2., -2.));
        assert_eq!(Vec2D::new(1, -1) * 2, Vec2D::new(2, -2));
    }

    #[test]
    fn vec2d_div() {
        assert_eq!(Vec2D::new(1., -1.) / 2., Vec2D::new(0.5, -0.5));
        assert_eq!(Vec2D::new(2, -1) / 2, Vec2D::new(1, 0));
    }
}
