use crate::css::{layout::CSSPixels, syntax::Token, values::Length, CSSParse, ParseError, Parser};

#[derive(Clone, Copy, Debug)]
pub enum LengthPercentage {
    Length(Length),
    Percent(f32),
}

impl LengthPercentage {
    pub const ZERO: Self = Self::Length(Length::ZERO);

    #[inline]
    #[must_use]
    pub fn resolve_against(&self, percent_of: CSSPixels) -> Length {
        match self {
            Self::Length(length) => *length,
            Self::Percent(p) => Length::from(percent_of * *p),
        }
    }
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

impl From<CSSPixels> for LengthPercentage {
    fn from(value: CSSPixels) -> Self {
        Self::Length(value.into())
    }
}
