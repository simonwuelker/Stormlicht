use string_interner::{static_interned, static_str};

use crate::css::{syntax::Token, CSSParse, ParseError, Parser, values::LengthPercentage};

#[derive(Clone, Copy, Debug)]
pub enum MarginValue {
    Auto,
    LengthPercentage(LengthPercentage)
}

impl<'a> CSSParse<'a> for MarginValue {
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        if parser.peek_token() == Some(Token::Ident(static_interned!("auto"))) {
            Ok(Self::Auto)
        } else {
            let length_percentage = LengthPercentage::parse(parser)?;
            Ok(Self::LengthPercentage(length_percentage))
        }
    }
}