use crate::css::{
    CSSParse, Parser, ParseError, values::Length, syntax::Token,
};

#[derive(Clone, Copy, Debug)]
pub enum LengthPercentage {
    Length(Length),
    Percent(f32),
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