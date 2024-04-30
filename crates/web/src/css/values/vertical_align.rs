//! <https://drafts.csswg.org/css2/#propdef-vertical-align>

use crate::{
    css::{syntax::Token, CSSParse, ParseError, Parser},
    static_interned, InternedString,
};

use super::{Length, Percentage};

#[derive(Clone, Debug)]
/// <https://drafts.csswg.org/css2/#propdef-vertical-align>
pub enum VerticalAlign {
    /// <https://drafts.csswg.org/css2/#valdef-vertical-align-baseline>
    Baseline,

    /// <https://drafts.csswg.org/css2/#valdef-vertical-align-sub>
    Sub,

    /// <https://drafts.csswg.org/css2/#valdef-vertical-align-super>
    Super,

    /// <https://drafts.csswg.org/css2/#valdef-vertical-align-top>
    Top,

    /// <https://drafts.csswg.org/css2/#valdef-vertical-align-text-top>
    TextTop,

    /// <https://drafts.csswg.org/css2/#valdef-vertical-align-middle>
    Middle,

    /// <https://drafts.csswg.org/css2/#valdef-vertical-align-bottom>
    Bottom,

    /// <https://drafts.csswg.org/css2/#valdef-vertical-align-text-bottom>
    TextBottom,
    Percentage(Percentage),
    Length(Length),
}

impl VerticalAlign {
    pub const fn from_name(name: InternedString) -> Result<Self, ParseError> {
        let vertical_align = match name {
            static_interned!("baseline") => Self::Baseline,
            static_interned!("sub") => Self::Sub,
            static_interned!("super") => Self::Super,
            static_interned!("top") => Self::Top,
            static_interned!("text-top") => Self::TextTop,
            static_interned!("middle") => Self::Middle,
            static_interned!("bottom") => Self::Bottom,
            static_interned!("text-bottom") => Self::TextBottom,
            _ => return Err(ParseError),
        };

        Ok(vertical_align)
    }
}

impl<'a> CSSParse<'a> for VerticalAlign {
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        let vertical_align = match parser.peek_token_ignoring_whitespace(0) {
            Some(Token::Ident(ident)) => {
                let align = Self::from_name(*ident)?;
                _ = parser.next_token_ignoring_whitespace();
                align
            },
            Some(Token::Percentage(p)) => {
                let align = Self::Percentage(Percentage::from_css_percentage(*p));
                _ = parser.next_token_ignoring_whitespace();

                align
            },
            Some(_) => {
                let length = parser.parse()?;
                Self::Length(length)
            },
            _ => return Err(ParseError),
        };

        Ok(vertical_align)
    }
}
