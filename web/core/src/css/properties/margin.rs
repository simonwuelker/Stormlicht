use string_interner::{static_interned, static_str};

use crate::css::{syntax::Token, values::LengthPercentage, CSSParse, ParseError, Parser};

#[derive(Clone, Copy, Debug)]
pub enum MarginValue {
    Auto,
    LengthPercentage(LengthPercentage),
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

impl MarginValue {
    #[must_use]
    pub fn resolve_auto(&self, resolve_auto_to: LengthPercentage) -> LengthPercentage {
        match self {
            Self::Auto => resolve_auto_to,
            Self::LengthPercentage(lp) => *lp,
        }
    }
}

impl Default for MarginValue {
    fn default() -> Self {
        Self::LengthPercentage(LengthPercentage::ZERO)
    }
}
