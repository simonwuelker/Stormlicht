use string_interner::{static_interned, static_str};

use crate::css::{syntax::Token, values::Length, CSSParse, ParseError, Parser};

/// <https://drafts.csswg.org/css2/#propdef-width>
#[derive(Clone, Copy, Debug)]
pub enum WidthValue {
    Auto,
    Percentage(f32),
    Lenght(Length),
}

impl WidthValue {
    #[must_use]
    pub fn is_auto(&self) -> bool {
        matches!(self, Self::Auto)
    }
}

impl Default for WidthValue {
    fn default() -> Self {
        Self::Auto
    }
}

impl<'a> CSSParse<'a> for WidthValue {
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        let width_value = match parser.peek_token() {
            Some(Token::Percentage(n)) => {
                parser.next_token();
                Self::Percentage(n.into())
            },
            Some(Token::Ident(static_interned!("auto"))) => {
                parser.next_token();
                Self::Auto
            },
            _ => Self::Lenght(Length::parse(parser)?),
        };
        parser.skip_whitespace();
        Ok(width_value)
    }
}
