//! <https://drafts.csswg.org/css2/#propdef-line-height>

use crate::{
    css::{layout::Pixels, syntax::Token, CSSParse, ParseError, Parser},
    static_interned,
};

use super::{length, Length, Number, Percentage};

#[derive(Clone, Debug)]
/// <https://drafts.csswg.org/css2/#propdef-line-height>
pub enum LineHeight {
    /// <https://drafts.csswg.org/css2/#valdef-line-height-normal>
    Normal,
    Number(Number),
    Percentage(Percentage),
    Length(Length),
}

impl LineHeight {
    pub fn to_pixels(&self, resolution_context: length::ResolutionContext) -> Pixels {
        match self {
            Self::Normal => resolution_context.font_size,
            Self::Length(length) => length.absolutize(resolution_context),
            Self::Percentage(p) => p.as_fraction() * resolution_context.font_size,
            Self::Number(n) => f32::from(*n) * resolution_context.font_size,
        }
    }
}

impl<'a> CSSParse<'a> for LineHeight {
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        let line_height = match parser.peek_token_ignoring_whitespace(0) {
            Some(Token::Ident(static_interned!("normal"))) => {
                _ = parser.next_token_ignoring_whitespace();

                Self::Normal
            },
            Some(Token::Percentage(p)) => {
                let line_height = Self::Percentage(Percentage::from_css_percentage(*p));
                _ = parser.next_token_ignoring_whitespace();

                line_height
            },
            Some(Token::Number(n)) => {
                let line_height = Self::Number(*n);
                _ = parser.next_token_ignoring_whitespace();

                line_height
            },
            Some(_) => {
                let length = parser.parse()?;
                Self::Length(length)
            },
            _ => return Err(ParseError),
        };

        Ok(line_height)
    }
}
