//! Implements the JS Number type according to https://262.ecma-international.org/13.0

use crate::{
    abstract_ops,
};

pub type IntegralNumber = i64;

#[derive(Debug, PartialOrd, PartialEq)]
pub struct Number(f64);

impl Number {
    pub const NAN: Self = Self(f64::NAN);
    pub const INFINITY: Self = Self(f64::INFINITY);
    pub const NEG_INFINITY: Self = Self(f64::NEG_INFINITY);
    pub const ONE: Self = Self(1.);
    pub const ZERO: Self = Self(0.);
    pub const NEG_ZERO: Self = Self(-0.);

    pub fn as_float(&self) -> f64 {
        self.0
    }

    pub fn is_nan(&self) -> bool {
        self.0.is_nan()
    }

    pub fn is_abs_zero(&self) -> bool {
        self.0 == 0.
    }

    pub fn is_positive(&self) -> bool {
        self.0 > 0.
    }

    pub fn is_negative(&self) -> bool {
        self.0 < -0.
    }

    pub fn is_finite(&self) -> bool {
        self.0.is_finite()
    }

    fn abs(self) -> Self { todo!() }

    // Spec functions start here

    /// The abstract operation [Number::unary_minus] takes argument x (a Number) and returns a Number.
    /// It performs the following steps when called:
    ///
    /// 1. If x is NaN, return NaN.
    /// 2. Return the result of negating x; that is, compute a Number with the same
    ///     magnitude but opposite sign.
    pub fn unary_minus(x: Self) -> Self {
        if x.is_nan() {
            Self::NAN
        } else {
            Self(-x.0)
        }
    }

    pub fn bitwise_not(x: Self) -> Self {
        let old = todo!("Let oldValue be ! ToInt32(x).");
    }

    pub fn exponentiate(base: Self, exponent: Self) -> Self {
        if exponent.is_nan() {
            Self::NAN
        } else if exponent.is_abs_zero() {
            Self::ONE
        } else if base.is_nan() {
            Self::NAN
        } else if base == Self::INFINITY {
            if exponent.is_positive() {
                Self::INFINITY
            } else {
                Self::ZERO
            }
        } else if base == Self::NEG_INFINITY {
            if exponent.is_positive() {
                if exponent.0 % 2. == 1. {
                    Self::NEG_INFINITY
                } else {
                    Self::INFINITY
                }
            } else {
                if exponent.0 % 2. == 1. {
                    Self::NEG_ZERO
                } else {
                    Self::ZERO
                }
            }
        } else if base == Self::ZERO {
            if exponent.is_positive() {
                Self::ZERO
            } else {
                Self::INFINITY
            }
        } else if base == Self::NEG_ZERO {
            if exponent.is_positive() {
                if (exponent.0 % 2.).abs() == 1. {
                    Self::NEG_ZERO
                } else {
                    Self::ZERO
                }
            } else {
                if (exponent.0 % 2.).abs() == 1. {
                    Self::NEG_INFINITY
                } else {
                    Self::INFINITY
                }
            }
        } else {
            assert!(base.is_finite());
            assert!(base != Self::ZERO);
            assert!(base != Self::NEG_ZERO);

            if exponent == Self::INFINITY {
                let absolute = Number::abs(base);
                if absolute > Self::ONE {
                    Self::INFINITY
                } else if absolute == Self::ONE {
                    Self::NAN
                } else {
                    Self::ZERO
                }
            } else if exponent == Self::NEG_INFINITY {
                let absolute = Number::abs(base);
                if absolute > Self::ONE {
                    Self::ZERO
                } else if absolute == Self::ONE {
                    Self::NAN
                } else {
                    Self::INFINITY
                }
            } else {
                assert!(exponent.is_finite());
                assert!(exponent != Self::ZERO);
                assert!(exponent != Self::NEG_ZERO);

                if base.is_negative() && exponent.0 % 1. != 0. {
                    Self::NAN
                } else {
                    Self(base.0.powf(exponent.0))
                }
            }
        }
    }

    pub fn multiply(x: Self, y: Self) -> Self {
        if x.is_nan() || y.is_nan() {
            Self::NAN
        } else if x == Self::INFINITY || x == Self::NEG_INFINITY {
            if y == Self::ZERO || y == Self::NEG_ZERO {
                Self::NAN
            } else if y.is_positive() {
                x
            } else {
                Number::unary_minus(x)
            }
        } else if y == Self::INFINITY || y == Self::NEG_INFINITY {
            if x == Self::ZERO || x == Self::NEG_ZERO {
                Self::NAN
            } else if x.is_positive() {
                y
            } else {
                Number::unary_minus(y)
            }
        } else if x == Self::NEG_ZERO {
            if y == Self::NEG_ZERO || y.is_negative() {
                Self::ZERO
            } else {
                Self::NEG_ZERO
            }
        } else if y == Self::NEG_ZERO {
            if x.is_negative() {
                Self::ZERO
            } else {
                Self::NEG_ZERO
            }
        } else {
            Self(x.0 * y.0)
        }
    }

    pub fn divide(x: Self, y: Self) -> Self {
        if x.is_nan() || y.is_nan() {
            Self::NAN
        } else if x == Self::INFINITY || x == Self::NEG_INFINITY {
            if y == Self::INFINITY || y == Self::NEG_INFINITY {
                Self::NAN
            } else if y == Self::ZERO || y.is_positive() {
                x
            } else {
                Number::unary_minus(x)
            }
        } else if y == Self::INFINITY {
            if x == Self::ZERO || x.is_positive() {
                Self::ZERO
            } else {
                Self::NEG_ZERO
            }
        } else if y == Self::NEG_INFINITY {
            if x == Self::ZERO || x.is_positive() {
                Self::NEG_ZERO
            } else {
                Self::ZERO
            }
        } else if x == Self::ZERO || x == Self::NEG_ZERO {
            if y == Self::ZERO || y == Self::NEG_ZERO {
                Self::NAN
            } else if y.is_positive() {
                x
            } else {
                Number::unary_minus(x)
            }
        } else if y == Self::ZERO {
            if x.is_positive() {
                Self::INFINITY
            } else {
                Self::NEG_INFINITY
            }
        } else if y == Self::NEG_ZERO {
            if x.is_positive() {
                Self::NEG_INFINITY
            } else {
                Self::INFINITY
            }
        } else {
            Self(x.0 / y.0)
        }
    }

    pub fn remainder(n: Self, d: Self) -> Self {
        if n.is_nan() || d.is_nan() {
            Self::NAN
        } else if !n.is_finite() {
            Self::NAN
        } else if !d.is_finite() {
            n
        } else if d.is_abs_zero() {
            Self::NAN
        } else if n.is_abs_zero() {
            n
        } else {
            assert!(n.is_finite());
            assert!(d.is_finite());
            assert!(!n.is_abs_zero());
            assert!(!d.is_abs_zero());
            Self(n.0 % d.0)
        }
    }

    pub fn add(x: Self, y: Self) -> Self {
        if x.is_nan() || y.is_nan() {
            Self::NAN
        } else if x == Self::INFINITY && y == Self::NEG_INFINITY {
            Self::NAN
        } else if x == Self::NEG_INFINITY && y == Self::INFINITY {
            Self::NAN
        } else if !x.is_finite() {
            x
        } else if !y.is_finite() {
            y
        } else {
            assert!(x.is_finite());
            assert!(y.is_finite());

            if x == Self::NEG_ZERO && y == Self::NEG_ZERO {
                Self::NEG_ZERO
            } else {
                Self(x.0 + y.0)
            }
        }
    }

    pub fn subtract(x: Self, y: Self) -> Self {
        Number::add(x, Number::unary_minus(y))
    }

    // pub fn left_shift(x: Self, y: Self) -> IntegralNumber {
    //     let lnum = abstract_ops.to_int32(x).unwrap();
    //     let shift_count = abstract_ops.to_uint32(y).unwrap() % 32;
    // }
}

#[cfg(test)]
mod tests {
    use super::Number;

    #[test]
    fn test_unary_minus() {
        assert!(Number::unary_minus(Number::NAN).is_nan());
        assert_eq!(Number::unary_minus(Number::ZERO), Number::NEG_ZERO);
        assert_eq!(Number::unary_minus(Number::NEG_ZERO), Number::ZERO);
        assert_eq!(Number::unary_minus(Number::INFINITY), Number::NEG_INFINITY);
        assert_eq!(Number::unary_minus(Number::NEG_INFINITY), Number::INFINITY);
        assert_eq!(Number::unary_minus(Number(3.5)), Number(-3.5));
    }

    #[test]
    fn test_exponentiate() {
        assert!(Number::exponentiate(Number::ONE, Number::NAN).is_nan());
        assert_eq!(Number::exponentiate(Number(2.5), Number::ZERO), Number::ONE);
        assert!(Number::exponentiate(Number::NAN, Number::ONE).is_nan());
        assert_eq!(
            Number::exponentiate(Number::NEG_INFINITY, Number(2.0)),
            Number::INFINITY
        );
        assert_eq!(
            Number::exponentiate(Number::NEG_INFINITY, Number(3.0)),
            Number::NEG_INFINITY
        );
        assert_eq!(
            Number::exponentiate(Number::NEG_INFINITY, Number(-2.0)),
            Number::ZERO
        );
        assert_eq!(
            Number::exponentiate(Number::NEG_INFINITY, Number(-3.0)),
            Number::NEG_ZERO
        );
        assert_eq!(
            Number::exponentiate(Number::ZERO, Number(2.0)),
            Number::ZERO
        );
        assert_eq!(
            Number::exponentiate(Number::ZERO, Number(-2.0)),
            Number::INFINITY
        );
        assert_eq!(
            Number::exponentiate(Number::NEG_ZERO, Number(2.0)),
            Number::ZERO
        );
        assert_eq!(
            Number::exponentiate(Number::NEG_ZERO, Number(3.0)),
            Number::NEG_ZERO
        );
        assert_eq!(
            Number::exponentiate(Number::NEG_ZERO, Number(-2.0)),
            Number::INFINITY
        );
        assert_eq!(
            Number::exponentiate(Number::NEG_ZERO, Number(-3.0)),
            Number::NEG_INFINITY
        );
        assert_eq!(
            Number::exponentiate(Number(0.5), Number::INFINITY),
            Number::ZERO
        );
        assert!(Number::exponentiate(Number(1.0), Number::INFINITY).is_nan());
        assert_eq!(
            Number::exponentiate(Number(1.5), Number::INFINITY),
            Number::INFINITY
        );
        assert_eq!(
            Number::exponentiate(Number(0.5), Number::NEG_INFINITY),
            Number::INFINITY
        );
        assert!(Number::exponentiate(Number(1.0), Number::NEG_INFINITY).is_nan());
        assert_eq!(
            Number::exponentiate(Number(1.5), Number::NEG_INFINITY),
            Number::ZERO
        );
        assert!(Number::exponentiate(Number(-1.5), Number(2.5)).is_nan());
    }
}
