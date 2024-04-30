use std::{fmt, ops::Mul};

use crate::css::{syntax::Token, CSSParse, ParseError, Parser};

use super::Number;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PercentageOr<T> {
    Percentage(Percentage),
    NotPercentage(T),
}

#[derive(Clone, Copy, PartialEq)]
pub struct Percentage(f32);

impl<T: Default> Default for PercentageOr<T> {
    fn default() -> Self {
        Self::NotPercentage(T::default())
    }
}

impl Percentage {
    pub const ZERO: Self = Self(0.);

    #[inline]
    #[must_use]
    pub fn from_css_percentage(n: Number) -> Self {
        let parsed_percentage: f32 = n.into();
        let fraction = parsed_percentage / 100.;

        Self::from_fraction(fraction)
    }

    #[inline]
    #[must_use]
    pub const fn from_fraction(fraction: f32) -> Self {
        Self(fraction)
    }

    #[inline]
    #[must_use]
    pub const fn as_fraction(&self) -> f32 {
        self.0
    }
}

impl fmt::Debug for Percentage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}%", self.as_fraction() * 100.)
    }
}

impl<T> PercentageOr<T>
where
    T: Mul<Percentage, Output = T>,
{
    #[inline]
    #[must_use]
    pub fn resolve_against(self, percent_of: T) -> T {
        match self {
            Self::NotPercentage(value) => value,
            Self::Percentage(p) => percent_of * p,
        }
    }
}

impl<'a, T> CSSParse<'a> for PercentageOr<T>
where
    T: CSSParse<'a>,
{
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        if let Some(Token::Percentage(n)) = parser.peek_token_ignoring_whitespace(0) {
            let percentage = Percentage::from_css_percentage(*n);
            let _ = parser.next_token_ignoring_whitespace();
            Ok(Self::Percentage(percentage))
        } else {
            let value = T::parse(parser)?;
            Ok(Self::NotPercentage(value))
        }
    }
}

impl<T> From<T> for PercentageOr<T> {
    fn from(value: T) -> Self {
        Self::NotPercentage(value)
    }
}

impl From<Number> for Percentage {
    fn from(value: Number) -> Self {
        match value {
            Number::Integer(i) => Self(i as f32),
            Number::Number(f) => Self(f),
        }
    }
}
