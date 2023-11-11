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
        match parser.peek_token() {
            Some(Token::Ident(static_interned!("transparent"))) => {
                parser.next_token();
                parser.skip_whitespace();
                Ok(Self::Transparent)
            },
            _ => Ok(Self::Color(Color::parse(parser)?)),
        }
    }
}
