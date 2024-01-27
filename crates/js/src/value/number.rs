#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Number(f64);

impl Number {
    pub const NAN: Self = Self(f64::NAN);
    pub const INFINITY: Self = Self(f64::INFINITY);
    pub const NEG_INFINITY: Self = Self(f64::NEG_INFINITY);
    pub const ZERO: Self = Self(0.);
    pub const NEG_ZERO: Self = Self(-0.);
    pub const ONE: Self = Self(1.);

    #[must_use]
    pub const fn new(value: f64) -> Self {
        Self(value)
    }

    #[must_use]
    pub fn is_nan(&self) -> bool {
        self.0.is_nan()
    }

    /// <https://262.ecma-international.org/14.0/#sec-numeric-types-number-tostring>
    #[must_use]
    pub fn to_string(&self, base: u8) -> String {
        _ = base;
        todo!()
    }

    /// <https://262.ecma-international.org/14.0/#sec-numeric-types-number-add>
    #[must_use]
    pub fn add(&self, other: Self) -> Self {
        Self(self.0 + other.0)
    }

    /// <https://262.ecma-international.org/#sec-numeric-types-number-equal>
    #[must_use]
    pub fn equal(x: Self, y: Self) -> bool {
        // 1. If x is NaN, return false.
        if x.is_nan() {
            return false;
        }

        // 2. If y is NaN, return false.
        if y.is_nan() {
            return false;
        }

        // 3. If x is y, return true.
        if x == y {
            return true;
        }

        // 4. If x is +0𝔽 and y is -0𝔽, return true.
        if x == Number::ZERO && y == Number::NEG_ZERO {
            return true;
        }

        // 5. If x is -0𝔽 and y is +0𝔽, return true.
        if x == Number::NEG_ZERO && y == Number::ZERO {
            return true;
        }

        // 6. Return false.
        return false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn number_add() {
        let any = Number::new(42.);
        assert!(Number::NAN.add(any).is_nan());
        assert!(any.add(Number::NAN).is_nan());

        assert!(Number::NEG_INFINITY.add(Number::INFINITY).is_nan());
        assert!(Number::INFINITY.add(Number::NEG_INFINITY).is_nan());

        assert_eq!(Number::INFINITY.add(any), Number::INFINITY);
        assert_eq!(Number::NEG_INFINITY.add(any), Number::NEG_INFINITY);
        assert_eq!(any.add(Number::INFINITY), Number::INFINITY);
        assert_eq!(any.add(Number::NEG_INFINITY), Number::NEG_INFINITY);

        assert_eq!(Number::NEG_ZERO.add(Number::NEG_ZERO), Number::NEG_ZERO);
    }
}
