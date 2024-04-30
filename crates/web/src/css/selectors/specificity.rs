use std::{
    iter::Sum,
    ops::{Add, AddAssign},
};

/// <https://drafts.csswg.org/selectors-4/#specificity-rules>
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Specificity {
    /// Number of ID selectors
    a: u8,

    /// Number of Class-, Attribute- and Pseudo Class Selectors
    b: u8,

    /// Number of Type- and Pseudo Element selectors
    c: u8,
}

impl Specificity {
    /// The specificity if no selector is present
    pub const ZERO: Self = Self::new(0, 0, 0);

    // The highst possible [Specificity]
    pub const MAX: Self = Self::new(u8::MAX, u8::MAX, u8::MAX);

    #[must_use]
    pub const fn new(a: u8, b: u8, c: u8) -> Self {
        Self { a, b, c }
    }
}

impl Add for Specificity {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            a: self.a.saturating_add(rhs.a),
            b: self.b.saturating_add(rhs.b),
            c: self.c.saturating_add(rhs.c),
        }
    }
}

impl AddAssign for Specificity {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sum<Specificity> for Specificity {
    fn sum<I: Iterator<Item = Specificity>>(iter: I) -> Self {
        let mut specificity = Self::ZERO;
        for s in iter {
            specificity += s;
        }
        specificity
    }
}

#[cfg(test)]
mod tests {
    use super::Specificity;

    #[test]
    fn order() {
        assert!(Specificity::new(3, 3, 3) > Specificity::new(2, 4, 4));
        assert!(Specificity::new(3, 3, 3) > Specificity::new(3, 2, 4));
        assert!(Specificity::new(3, 3, 3) > Specificity::new(3, 3, 2));
        assert_eq!(Specificity::new(3, 3, 3), Specificity::new(3, 3, 3));
    }
}
