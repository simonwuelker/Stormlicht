//! Fixed-point arithmetic

use std::{fmt, ops};

/// Fixed point floating number using two's complement
///
/// `N` defines the number of fractional bits. The number of
/// integer bits is equal to `32 - N`.
///
/// # Examples
///
/// Basic Usage:
/// ```
/// # use sl_std::fixed::Fixed;
/// let a = Fixed::<5>::from(1.5);
/// let b = Fixed::<5>::from(3.2);
///
/// assert_eq!(a + b, Fixed::<5>::from(4.7));
/// ```
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct Fixed<const N: usize>(i32);

impl<const N: usize> Fixed<N> {
    pub const INT_BITS: usize = 32 - N;
    pub const FRAC_BITS: usize = N;

    // Option::unwrap_or(0) is not const :/
    const SCALING_FACTOR: usize = match 1_usize.checked_shl(Self::FRAC_BITS as u32) {
        Some(v) => v,
        None => 0,
    };

    /// Computes the absolute value of `self`.
    ///
    /// # Examples
    ///
    /// Basic Usage:
    /// ```
    /// # use sl_std::fixed::Fixed;
    /// assert_eq!(Fixed::<3>::from(42.).abs(), Fixed::<3>::from(42.));
    /// assert_eq!(Fixed::<3>::from(-42.).abs(), Fixed::<3>::from(42.));
    /// ```
    #[inline]
    #[must_use]
    pub const fn abs(&self) -> Self {
        Self(self.0.abs())
    }

    #[must_use]
    pub const fn bits(&self) -> i32 {
        self.0
    }

    #[must_use]
    pub const fn from_bits(bits: i32) -> Self {
        Self(bits)
    }

    /// Returns the smallest integer greater than or equal to `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sl_std::fixed::Fixed;
    /// let f = Fixed::<5>::from(3.0);
    /// let g = Fixed::<5>::from(3.9);
    ///
    /// assert_eq!(f.floor(), Fixed::<5>::from(3.0));
    /// assert_eq!(g.floor(), Fixed::<5>::from(3.0));
    /// ```
    #[must_use]
    #[inline]
    pub const fn floor(&self) -> Self {
        let mask = (1 << Self::FRAC_BITS) - 1;
        Self(self.0 & !mask)
    }
}

impl<const N: usize> From<f32> for Fixed<N> {
    fn from(value: f32) -> Self {
        let bits = (value * Self::SCALING_FACTOR as f32).round() as i32;
        Self(bits)
    }
}

impl<const N: usize> From<Fixed<N>> for f32 {
    fn from(value: Fixed<N>) -> Self {
        value.bits() as f32 / Fixed::<N>::SCALING_FACTOR as f32
    }
}

impl<const N: usize> fmt::Debug for Fixed<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", f32::from(*self))?;
        Ok(())
    }
}

impl<const N: usize> fmt::Display for Fixed<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", f32::from(*self))
    }
}

impl<const N: usize> fmt::Binary for Fixed<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:b}", self.bits())
    }
}

impl<const N: usize> fmt::Octal for Fixed<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:o}", self.bits())
    }
}

impl<const N: usize> fmt::LowerHex for Fixed<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:x}", self.bits())
    }
}

impl<const N: usize> fmt::UpperHex for Fixed<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:X}", self.bits())
    }
}

impl<const N: usize> ops::Add for Fixed<N> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl<const N: usize> ops::AddAssign for Fixed<N> {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl<const N: usize> ops::Sub for Fixed<N> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl<const N: usize> ops::SubAssign for Fixed<N> {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl<const N: usize> ops::Shl<usize> for Fixed<N> {
    type Output = Self;

    fn shl(self, rhs: usize) -> Self::Output {
        Self(self.0.shl(rhs))
    }
}

impl<const N: usize> ops::Shr<usize> for Fixed<N> {
    type Output = Self;

    fn shr(self, rhs: usize) -> Self::Output {
        Self(self.0.shr(rhs))
    }
}

impl<const N: usize> PartialOrd for Fixed<N> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<const N: usize> Ord for Fixed<N> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

pub trait FixedNum:
    private::Sealed
    + From<f32>
    + Into<f32>
    + ops::Add<Output = Self>
    + ops::AddAssign
    + ops::Sub<Output = Self>
    + ops::SubAssign
    + ops::Shl<usize, Output = Self>
    + ops::Shr<usize, Output = Self>
    + PartialEq
    + Eq
    + PartialOrd
    + Ord
    + fmt::Debug
    + fmt::Display
    + fmt::Binary
    + fmt::Octal
    + fmt::LowerHex
    + fmt::UpperHex
{
}

impl<const N: usize> FixedNum for Fixed<N> {}

mod private {
    use super::Fixed;

    pub trait Sealed {}

    impl<const N: usize> Sealed for Fixed<N> {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn float_conversion() {
        fn float_conversion_with_size<F: FixedNum>() {
            let test = |float: f32| {
                let roundtrip: f32 = F::from(float).into();
                assert_eq!(roundtrip, float);
            };

            test(0.);
            test(-0.);
            test(1.25);
            test(-1.75);
        }

        float_conversion_with_size::<Fixed<2>>();
        float_conversion_with_size::<Fixed<7>>();
        float_conversion_with_size::<Fixed<16>>();
    }

    #[test]
    fn add() {
        fn add_with_size<F: FixedNum>() {
            assert_eq!(F::from(1.) + F::from(2.), F::from(3.));
            assert_eq!(F::from(1.5) + F::from(0.25), F::from(1.75));
            assert_eq!(F::from(-4.75) + F::from(1.5), F::from(-3.25));
            assert_eq!(F::from(-1.25) + F::from(-1.25), F::from(-2.5));
        }

        add_with_size::<Fixed<2>>();
        add_with_size::<Fixed<7>>();
        add_with_size::<Fixed<16>>();
    }

    #[test]
    fn sub() {
        fn sub_with_size<F: FixedNum>() {
            assert_eq!(F::from(1.) - F::from(2.), F::from(-1.));
            assert_eq!(F::from(1.5) - F::from(0.25), F::from(1.25));
            assert_eq!(F::from(-4.75) - F::from(1.5), F::from(-6.25));
            assert_eq!(F::from(-1.25) - F::from(-1.25), F::from(0.));
        }

        sub_with_size::<Fixed<2>>();
        sub_with_size::<Fixed<7>>();
        sub_with_size::<Fixed<16>>();
    }

    #[test]
    fn ordering() {
        fn ordering_with_size<F: FixedNum>() {
            use std::cmp::Ordering;

            assert_eq!(F::from(1.25).cmp(&F::from(3.75)), Ordering::Less);
            assert_eq!(F::from(1.25).cmp(&F::from(-3.75)), Ordering::Greater);
        }

        ordering_with_size::<Fixed<2>>();
        ordering_with_size::<Fixed<7>>();
        ordering_with_size::<Fixed<16>>();
    }

    #[test]
    fn floor() {
        type F = Fixed<2>;
        assert_eq!(F::from(1.25).floor(), F::from(1.));
        assert_eq!(F::from(-1.75).floor(), F::from(-2.));
    }

    #[test]
    fn abs() {
        type F = Fixed<2>;
        assert_eq!(F::from(1.25).abs(), F::from(1.25));
        assert_eq!(F::from(-1.75).abs(), F::from(1.75));
    }
}
