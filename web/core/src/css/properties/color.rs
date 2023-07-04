use crate::css::{syntax::Token, values::color::Color, CSSParse, ParseError, Parser};
use string_interner::{static_interned, static_str};

/// <https://drafts.csswg.org/css2/#colors>
#[derive(Clone, Copy, Debug)]
pub enum ColorValue {
    Color(Color),
    Inherit,
}

impl<'a> CSSParse<'a> for ColorValue {
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        if parser.peek_token() == Some(Token::Ident(static_interned!("inherit"))) {
            parser.next_token();
            parser.skip_whitespace();
            Ok(Self::Inherit)
        } else {
            Ok(Self::Color(Color::parse(parser)?))
        }
    }
}
