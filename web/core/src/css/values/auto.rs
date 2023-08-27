use string_interner::{static_interned, static_str};

use crate::css::{syntax::Token, CSSParse, ParseError, Parser};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AutoOr<T> {
    Auto,
    NotAuto(T),
}

impl<T> AutoOr<T> {
    #[inline]
    #[must_use]
    pub fn is_auto(&self) -> bool {
        matches!(self, Self::Auto)
    }

    /// Maps an `AutoOr<T>` to `AutoOr<U>` by applying a function to a contained value (if `NotAuto`) or returns `Auto` (if `Auto`).
    ///
    /// # Examples
    ///
    /// Resolves a `LengthPercentage` inside `AutoOr`, consuming the original:
    /// ```rust
    /// # use core::css::{layout::CSSPixels, values::{LengthPercentage, AutoOr}};
    /// let maybe_lengthpercent = AutoOr::NotAuto(LengthPercentage::Percent(0.2));
    /// // `Option::map` takes self *by value*, consuming `maybe_some_string`
    /// let maybe_length = maybe_lengthpercent.map(|lp|
    ///     lp.resolve_against(CSSPixels(100.)).absolutize()
    /// );
    /// assert_eq!(maybe_length, AutoOr::NotAuto(CSSPixels(20.)));
    ///
    /// let x: AutoOr<LengthPercentage> = AutoOr::Auto;
    /// assert_eq!(
    ///     x.map(|lp| lp.resolve_against(CSSPixels(100.)).absolutize()),
    ///     AutoOr::Auto
    /// );
    /// ```
    #[inline]
    pub fn map<U, F>(self, f: F) -> AutoOr<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Self::Auto => AutoOr::Auto,
            Self::NotAuto(x) => AutoOr::NotAuto(f(x)),
        }
    }
}

impl<T> Default for AutoOr<T> {
    fn default() -> Self {
        Self::Auto
    }
}

impl<'a, T> CSSParse<'a> for AutoOr<T>
where
    T: CSSParse<'a>,
{
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        match parser.peek_token() {
            Some(Token::Ident(static_interned!("auto"))) => {
                parser.next_token();
                parser.skip_whitespace();
                Ok(Self::Auto)
            },
            _ => {
                let parsed_value = T::parse(parser)?;
                Ok(Self::NotAuto(parsed_value))
            },
        }
    }
}
