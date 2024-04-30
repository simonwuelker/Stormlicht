//! <https://drafts.csswg.org/css2/#propdef-line-height>

use crate::{
    css::{syntax::Token, CSSParse, ParseError, Parser},
    static_interned,
};

use super::{Length, Number, Percentage};

#[derive(Clone, Debug)]
/// <https://drafts.csswg.org/css2/#propdef-line-height>
pub enum LineHeight {
    /// <https://drafts.csswg.org/css2/#valdef-line-height-normal>
    Normal,
    Number(Number),
    Percentage(Percentage),
    Length(Length),
}

impl<'a> CSSParse<'a> for LineHeight {
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        let line_height = match parser.peek_token_ignoring_whitespace(0) {
            Some(Token::Ident(static_interned!("normal"))) => {
                _ = parser.next_token_ignoring_whitespace();

                Self::Normal
            },
            Some(Token::Percentage(p)) => {
                let line_height = Self::Percentage(Percentage::from_css_percentage(*p));
                _ = parser.next_token_ignoring_whitespace();

                line_height
            },
            Some(Token::Number(n)) => {
                let line_height = Self::Number(*n);
                _ = parser.next_token_ignoring_whitespace();

                line_height
            },
            Some(_) => {
                let length = parser.parse()?;
                Self::Length(length)
            },
            _ => return Err(ParseError),
        };

        Ok(line_height)
    }
}
