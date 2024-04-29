use crate::{
    css::{syntax::Token, values::color::Color, CSSParse, ParseError, Parser},
    static_interned,
};

/// <https://drafts.csswg.org/css2/#background-properties>
#[derive(Clone, Copy, Debug, Default)]
pub enum BackgroundColor {
    Color(Color),
    #[default]
    Transparent,
}

impl<'a> CSSParse<'a> for BackgroundColor {
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        match parser.peek_token_ignoring_whitespace() {
            Some(Token::Ident(static_interned!("transparent"))) => {
                let _ = parser.next_token_ignoring_whitespace();
                Ok(Self::Transparent)
            },
            _ => Ok(Self::Color(Color::parse(parser)?)),
        }
    }
}

impl From<Color> for BackgroundColor {
    fn from(value: Color) -> Self {
        Self::Color(value)
    }
}
