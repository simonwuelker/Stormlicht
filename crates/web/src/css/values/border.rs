use crate::{
    css::{layout::Pixels, syntax::Token, CSSParse, ParseError, Parser},
    static_interned, InternedString,
};

use super::{Color, Length};

/// <https://drafts.csswg.org/css-backgrounds/#typedef-line-style>
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LineStyle {
    /// <https://drafts.csswg.org/css-backgrounds/#valdef-line-style-none>
    None,

    /// <https://drafts.csswg.org/css-backgrounds/#valdef-line-style-hidden>
    Hidden,

    /// <https://drafts.csswg.org/css-backgrounds/#valdef-line-style-dotted>
    Dotted,

    /// <https://drafts.csswg.org/css-backgrounds/#valdef-line-style-dashed>
    Dashed,

    /// <https://drafts.csswg.org/css-backgrounds/#valdef-line-style-solid>
    #[default]
    Solid,

    /// <https://drafts.csswg.org/css-backgrounds/#valdef-line-style-double>
    Double,

    /// https://drafts.csswg.org/css-backgrounds/#valdef-line-style-groove>
    Groove,

    /// <https://drafts.csswg.org/css-backgrounds/#valdef-line-style-ridge>
    Ridge,

    /// <https://drafts.csswg.org/css-backgrounds/#valdef-line-style-inset>
    Inset,

    /// <https://drafts.csswg.org/css-backgrounds/#valdef-line-style-outset>
    Outset,
}

impl LineStyle {
    #[must_use]
    pub const fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    pub fn from_name(name: InternedString) -> Result<Self, ParseError> {
        let line_style = match name {
            static_interned!("none") => Self::None,
            static_interned!("hidden") => Self::Hidden,
            static_interned!("dotted") => Self::Dotted,
            static_interned!("dashed") => Self::Dashed,
            static_interned!("solid") => Self::Solid,
            static_interned!("double") => Self::Double,
            static_interned!("groove") => Self::Groove,
            static_interned!("ridge") => Self::Ridge,
            static_interned!("inset") => Self::Inset,
            static_interned!("outset") => Self::Outset,
            _ => return Err(ParseError),
        };

        Ok(line_style)
    }
}

impl<'a> CSSParse<'a> for LineStyle {
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        match parser.next_token_ignoring_whitespace() {
            Some(Token::Ident(name)) => Self::from_name(name),
            _ => Err(ParseError),
        }
    }
}

/// <https://drafts.csswg.org/css-backgrounds/#typedef-line-width>
#[derive(Clone, Copy, Debug)]
pub struct LineWidth(Length);

impl Default for LineWidth {
    fn default() -> Self {
        Self::MEDIUM
    }
}

impl LineWidth {
    pub const THIN: Self = Self(Length::pixels(Pixels(1.)));
    pub const MEDIUM: Self = Self(Length::pixels(Pixels(3.)));
    pub const THICK: Self = Self(Length::pixels(Pixels(5.)));

    pub const fn length(&self) -> Length {
        self.0
    }
}

impl LineWidth {
    fn from_name(name: InternedString) -> Result<Self, ParseError> {
        let line_width = match name {
            static_interned!("thin") => Self::THIN,
            static_interned!("medium") => Self::MEDIUM,
            static_interned!("thick") => Self::THICK,
            _ => return Err(ParseError),
        };

        Ok(line_width)
    }
}

impl<'a> CSSParse<'a> for LineWidth {
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        match parser.peek_token_ignoring_whitespace(0) {
            Some(Token::Ident(name)) => {
                let width = Self::from_name(*name)?;
                let _ = parser.next_token_ignoring_whitespace();

                Ok(width)
            },
            _ => {
                let length: Length = parser.parse()?;
                Ok(Self(length))
            },
        }
    }
}

/// The value of the CSS `border` property
#[derive(Clone, Copy, Debug)]
pub struct Border {
    pub color: Color,
    pub width: LineWidth,
    pub style: LineStyle,
}

impl<'a> CSSParse<'a> for Border {
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        let mut border_color = None;
        let mut border_width = None;
        let mut border_style = None;

        for _ in 0..3 {
            match parser.peek_token_ignoring_whitespace(0) {
                Some(Token::Hash(..) | Token::Function(_)) => {
                    border_color = Some(parser.parse()?);
                },
                Some(Token::Ident(
                    name @ (static_interned!("thin")
                    | static_interned!("medium")
                    | static_interned!("thick")),
                )) => {
                    border_width = Some(LineWidth::from_name(*name)?);
                    _ = parser.next_token_ignoring_whitespace();
                },
                Some(Token::Ident(
                    name @ (static_interned!("none")
                    | static_interned!("hidden")
                    | static_interned!("dotted")
                    | static_interned!("dashed")
                    | static_interned!("solid")
                    | static_interned!("double")
                    | static_interned!("groove")
                    | static_interned!("ridge")
                    | static_interned!("inset")
                    | static_interned!("outset")),
                )) => {
                    let style = LineStyle::from_name(*name)?;
                    _ = parser.next_token_ignoring_whitespace();

                    border_style = Some(style);
                },
                Some(Token::Dimension(value, unit_name)) => {
                    let length = Length::from_dimension(*value, *unit_name)?;
                    _ = parser.next_token_ignoring_whitespace();

                    border_width = Some(length.into());
                },
                Some(Token::Number(n)) if n.is_zero() => {
                    _ = parser.next_token_ignoring_whitespace();

                    border_width = Some(Length::ZERO.into());
                },
                Some(Token::Ident(other)) => {
                    border_color = Some(Color::from_name(*other)?);
                    _ = parser.next_token_ignoring_whitespace();
                },
                _ => {
                    // Could not make progress
                    break;
                },
            }
        }

        // If we didn't parse anything, that's an error
        if border_color.is_none() && border_width.is_none() && border_style.is_none() {
            return Err(ParseError);
        }

        let border = Border {
            color: border_color.unwrap_or(Color::BLACK), // FIXME: should be "currentcolor",
            width: border_width.unwrap_or_default(),
            style: border_style.unwrap_or_default(),
        };

        Ok(border)
    }
}

impl From<Length> for LineWidth {
    fn from(value: Length) -> Self {
        Self(value)
    }
}
