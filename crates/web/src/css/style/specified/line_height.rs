//! <https://drafts.csswg.org/css2/#propdef-line-height>

use crate::{
    css::{
        style::{computed, StyleContext, ToComputedStyle},
        syntax::Token,
        values::{Number, Percentage},
        CSSParse, ParseError, Parser,
    },
    static_interned,
};

use super::Length;

/// <https://drafts.csswg.org/css2/#propdef-line-height>
#[derive(Clone, Debug)]
pub enum LineHeight {
    /// <https://drafts.csswg.org/css2/#valdef-line-height-normal>
    Normal,
    Number(Number),
    Percentage(Percentage),
    Length(Length),
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

impl ToComputedStyle for LineHeight {
    type Computed = computed::LineHeight;

    fn to_computed_style(&self, context: &StyleContext) -> Self::Computed {
        match self {
            Self::Normal => Self::Computed::Normal,
            Self::Length(length) => {
                let computed_height = length.to_computed_style(context);
                Self::Computed::Absolute(computed_height)
            },
            Self::Percentage(p) => {
                let computed_height = p.as_fraction() * context.font_size;
                Self::Computed::Absolute(computed_height)
            },
            Self::Number(n) => Self::Computed::Relative(*n),
        }
    }
}
