use crate::{
    css::{layout::CSSPixels, syntax::Token, CSSParse, ParseError, Parser},
    static_interned,
};

use super::Length;

/// <https://drafts.csswg.org/css-backgrounds/#typedef-line-style>
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

impl LineWidth {
    pub const THIN: Self = Self(Length::pixels(CSSPixels(1.)));
    pub const MEDIUM: Self = Self(Length::pixels(CSSPixels(3.)));
    pub const THICK: Self = Self(Length::pixels(CSSPixels(5.)));
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
