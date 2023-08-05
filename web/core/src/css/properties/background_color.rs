use crate::css::{syntax::Token, values::color::Color, CSSParse, ParseError, Parser};
use string_interner::{static_interned, static_str};

/// <https://drafts.csswg.org/css2/#background-properties>
#[derive(Clone, Copy, Debug)]
pub enum BackgroundColorValue {
    Color(Color),
    Transparent,
    Inherit,
}

impl<'a> CSSParse<'a> for BackgroundColorValue {
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        match parser.peek_token() {
            Some(Token::Ident(static_interned!("inherit"))) => {
                parser.next_token();
                parser.skip_whitespace();
                Ok(Self::Inherit)
            },
            Some(Token::Ident(static_interned!("transparent"))) => {
                parser.next_token();
                parser.skip_whitespace();
                Ok(Self::Transparent)
            },
            _ => Ok(Self::Color(Color::parse(parser)?)),
        }
    }
}