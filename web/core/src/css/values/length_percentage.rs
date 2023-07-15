use crate::css::{syntax::Token, values::Length, CSSParse, ParseError, Parser};

#[derive(Clone, Copy, Debug)]
pub enum LengthPercentage {
    Length(Length),
    Percent(f32),
}

impl LengthPercentage {
    pub const ZERO: Self = Self::Length(Length::ZERO);
}

impl<'a> CSSParse<'a> for LengthPercentage {
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        if let Some(Token::Percentage(n)) = parser.peek_token() {
            parser.next_token();
            parser.skip_whitespace();
            Ok(Self::Percent(n.into()))
        } else {
            let length = Length::parse(parser)?;
            Ok(Self::Length(length))
        }
    }
}

impl From<Length> for LengthPercentage {
    fn from(value: Length) -> Self {
        Self::Length(value)
    }
}
