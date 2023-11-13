use crate::{
    css::{layout::CSSPixels, syntax::Token, CSSParse, ParseError, Parser},
    static_interned,
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
    pub const fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
}

impl<'a> CSSParse<'a> for LineStyle {
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        let line_style = match parser.next_token() {
            Some(Token::Ident(static_interned!("none"))) => Self::None,
            Some(Token::Ident(static_interned!("hidden"))) => Self::Hidden,
            Some(Token::Ident(static_interned!("dotted"))) => Self::Dotted,
            Some(Token::Ident(static_interned!("dashed"))) => Self::Dashed,
            Some(Token::Ident(static_interned!("solid"))) => Self::Solid,
            Some(Token::Ident(static_interned!("double"))) => Self::Double,
            Some(Token::Ident(static_interned!("groove"))) => Self::Groove,
            Some(Token::Ident(static_interned!("ridge"))) => Self::Ridge,
            Some(Token::Ident(static_interned!("inset"))) => Self::Inset,
            Some(Token::Ident(static_interned!("outset"))) => Self::Outset,
            _ => return Err(ParseError),
        };

        Ok(line_style)
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
    pub const THIN: Self = Self(Length::pixels(CSSPixels(1.)));
    pub const MEDIUM: Self = Self(Length::pixels(CSSPixels(3.)));
    pub const THICK: Self = Self(Length::pixels(CSSPixels(5.)));

    pub const fn length(&self) -> Length {
        self.0
    }
}

impl<'a> CSSParse<'a> for LineWidth {
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        match parser.peek_token() {
            Some(Token::Ident(static_interned!("thin"))) => {
                parser.next_token();
                Ok(Self::THIN)
            },
            Some(Token::Ident(static_interned!("medium"))) => {
                parser.next_token();
                Ok(Self::MEDIUM)
            },
            Some(Token::Ident(static_interned!("thick"))) => {
                parser.next_token();
                Ok(Self::THICK)
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
            if let Some(color) = parser.parse_optional()
                && border_color.is_none()
            {
                border_color = Some(color);
                parser.skip_whitespace();
                continue;
            }

            if let Some(width) = parser.parse_optional()
                && border_width.is_none()
            {
                border_width = Some(width);
                parser.skip_whitespace();
                continue;
            }

            if let Some(style) = parser.parse_optional()
                && border_style.is_none()
            {
                border_style = Some(style);
                parser.skip_whitespace();
                continue;
            }

            // Could not make progress
            break;
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
