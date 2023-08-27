use std::ops::Mul;

use crate::css::{syntax::Token, CSSParse, ParseError, Parser};

use super::Number;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PercentageOr<T> {
    Percentage(Percentage),
    NotPercentage(T),
}

#[derive(Clone, Copy, Debug, PartialEq)]
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
    pub fn from_fraction(fraction: f32) -> Self {
        Self(fraction)
    }

    #[inline]
    #[must_use]
    pub fn as_fraction(&self) -> f32 {
        self.0
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
        if let Some(Token::Percentage(n)) = parser.peek_token() {
            parser.next_token();
            parser.skip_whitespace();
            Ok(Self::Percentage(n.into()))
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
