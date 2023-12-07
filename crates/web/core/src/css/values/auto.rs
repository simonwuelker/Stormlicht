use crate::{
    css::{syntax::Token, CSSParse, ParseError, Parser},
    static_interned,
};

/// Represents a value that can either be `Auto` or something else
///
/// As this type is very similar to [Option], most methods and documentation
/// take heavy inspiration from it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AutoOr<T> {
    Auto,
    NotAuto(T),
}

impl<T> AutoOr<T> {
    #[inline]
    #[must_use]
    pub const fn is_auto(&self) -> bool {
        matches!(self, Self::Auto)
    }

    /// Converts from `&AutoOr<T>` to `AutoOr<&T>`.
    #[inline]
    pub const fn as_ref(&self) -> AutoOr<&T> {
        match *self {
            Self::NotAuto(ref x) => AutoOr::NotAuto(x),
            Self::Auto => AutoOr::Auto,
        }
    }

    /// Returns the `AutoOr` if it contains a value, otherwise returns `b`.
    ///
    /// Arguments passed to `or` are eagerly evaluated; if you are passing the
    /// result of a function call, it is recommended to use [`or_else`], which is
    /// lazily evaluated.
    ///
    /// [`or_else`]: Option::or_else
    ///
    /// # Examples
    ///
    /// ```
    /// # use core::css::values::AutoOr;
    /// let x = AutoOr::NotAuto(2);
    /// let y = AutoOr::Auto;
    /// assert_eq!(x.or(y), AutoOr::NotAuto(2));
    ///
    /// let x = AutoOr::Auto;
    /// let y = AutoOr::NotAuto(100);
    /// assert_eq!(x.or(y), AutoOr::NotAuto(100));
    ///
    /// let x = AutoOr::NotAuto(2);
    /// let y = AutoOr::NotAuto(100);
    /// assert_eq!(x.or(y), AutoOr::NotAuto(2));
    ///
    /// let x: AutoOr<u32> = AutoOr::Auto;
    /// let y = AutoOr::Auto;
    /// assert_eq!(x.or(y), AutoOr::Auto);
    /// ```
    #[inline]
    pub fn or(self, b: Self) -> Self {
        match self {
            Self::NotAuto(x) => Self::NotAuto(x),
            Self::Auto => b,
        }
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

    /// Returns the contained [`AutoOr::NotAuto`] value or a provided default.
    ///
    /// Arguments passed to `unwrap_or` are eagerly evaluated.
    ///
    /// # Examples
    ///
    /// ```
    /// # use core::css::values::AutoOr;
    /// assert_eq!(AutoOr::NotAuto(1).unwrap_or(0), 1);
    /// assert_eq!(AutoOr::Auto.unwrap_or(0), 0);
    /// ```
    #[inline]
    pub fn unwrap_or(self, default: T) -> T {
        match self {
            Self::NotAuto(x) => x,
            Self::Auto => default,
        }
    }

    /// Returns the contained [`AutoOr::NotAuto`] value or a default.
    ///
    /// Consumes the `self` argument then, if [`AutoOr::NotAuto`], returns the contained
    /// value, otherwise if [`AutoOr::Auto`], returns the [default value] for that
    /// type.
    ///
    /// # Examples
    ///
    /// ```
    /// # use core::css::values::AutoOr;
    /// let x: AutoOr<u32> = AutoOr::Auto;
    /// let y: AutoOr<u32> = AutoOr::NotAuto(12);
    ///
    /// assert_eq!(x.unwrap_or_default(), 0);
    /// assert_eq!(y.unwrap_or_default(), 12);
    /// ```
    #[inline]
    pub fn unwrap_or_default(self) -> T
    where
        T: Default,
    {
        match self {
            Self::NotAuto(x) => x,
            Self::Auto => T::default(),
        }
    }

    /// Returns `true` if the `AutoOr` is a [`AutoOr::NotAuto`] and the value inside of it matches a predicate.
    ///
    /// # Examples
    ///
    /// ```
    /// # use core::css::values::AutoOr;
    /// let x: AutoOr<u32> = AutoOr::NotAuto(2);
    /// assert_eq!(x.is_not_auto_and(|x| x > 1), true);
    ///
    /// let x: AutoOr<u32> = AutoOr::NotAuto(0);
    /// assert_eq!(x.is_not_auto_and(|x| x > 1), false);
    ///
    /// let x: AutoOr<u32> = AutoOr::Auto;
    /// assert_eq!(x.is_not_auto_and(|x| x > 1), false);
    /// ```
    #[inline]
    #[must_use]
    pub fn is_not_auto_and(&self, f: impl FnOnce(&T) -> bool) -> bool {
        match self {
            Self::Auto => false,
            Self::NotAuto(x) => f(x),
        }
    }

    /// Returns [`AutoOr::Auto`] if the value is [`AutoOr::Auto`], otherwise calls `f` with the
    /// wrapped value and returns the result.
    #[inline]
    pub fn flat_map<U, F>(self, f: F) -> AutoOr<U>
    where
        F: FnOnce(T) -> AutoOr<U>,
    {
        match self {
            AutoOr::NotAuto(x) => f(x),
            AutoOr::Auto => AutoOr::Auto,
        }
    }

    /// Returns the contained [`NotAuto`] value or computes it from a closure.
    #[inline]
    pub fn unwrap_or_else<F>(self, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        match self {
            AutoOr::NotAuto(x) => x,
            AutoOr::Auto => f(),
        }
    }

    #[must_use]
    pub fn into_option(self) -> Option<T> {
        match self {
            AutoOr::NotAuto(value) => Some(value),
            AutoOr::Auto => None,
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
