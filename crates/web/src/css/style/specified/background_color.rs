//! <https://drafts.csswg.org/css-backgrounds/#background-color>

use crate::{
    css::{
        style::{StyleContext, ToComputedStyle},
        syntax::Token,
        values::Color,
        CSSParse, ParseError, Parser,
    },
    static_interned,
};

/// <https://drafts.csswg.org/css-backgrounds/#background-color>
#[derive(Clone, Copy, Debug, Default)]
pub enum BackgroundColor {
    Color(Color),
    #[default]
    Transparent,
}

impl<'a> CSSParse<'a> for BackgroundColor {
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        match parser.peek_token_ignoring_whitespace(0) {
            Some(Token::Ident(static_interned!("transparent"))) => {
                let _ = parser.next_token_ignoring_whitespace();
                Ok(Self::Transparent)
            },
            _ => Ok(Self::Color(Color::parse(parser)?)),
        }
    }
}

impl ToComputedStyle for BackgroundColor {
    type Computed = Self;

    fn to_computed_style(&self, context: StyleContext) -> Self::Computed {
        _ = context;

        *self
    }
}

impl From<Color> for BackgroundColor {
    fn from(value: Color) -> Self {
        Self::Color(value)
    }
}
