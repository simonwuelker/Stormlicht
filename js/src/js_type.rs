pub enum JSType {
    Undefined,
    Null,
    Boolean(bool),
    String(String),
    // TODO: Symbol type
    Number(Number),
}

#[derive(PartialOrd, PartialEq)]
pub struct Number(f64);

impl Number {
    const NAN: Self = Self(f64::NAN);
    const INFINITY: Self = Self(f64::INFINITY);
    const NEG_INFINITY: Self = Self(f64::NEG_INFINITY);
    const ONE: Self = Self(1.);
    const ZERO: Self = Self(0.);
    const NEG_ZERO: Self = Self(-0.);

    fn is_nan(&self) -> bool {
        self.0.is_nan()
    }

    fn is_abs_zero(&self) -> bool {
        self.0 == 0.
    }

    fn is_positive(&self) -> bool {
        self.0 > 0.
    }

    fn is_negative(&self) -> bool {
        self.0 < -0.
    }

    fn is_finite(&self) -> bool {
        self.0.is_finite()
    }

    fn abs(x: Self) -> Self {
        todo!();
    }

    // Spec functions start here

    /// The abstract operation [Number::unary_minus] takes argument x (a Number) and returns a Number.
    /// It performs the following steps when called:
    ///
    ///     1. If x is NaN, return NaN.
    ///     2. Return the result of negating x; that is, compute a Number with the same
    ///            magnitude but opposite sign.
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
                if exponent.0 % 2. == 1. {
                    Self::NEG_ZERO
                } else {
                    Self::ZERO
                }
            } else {
                if exponent.0 % 2. == 1. {
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
            if (x.is_negative()) {
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
}
