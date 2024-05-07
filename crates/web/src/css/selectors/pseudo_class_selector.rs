//! <https://drafts.csswg.org/selectors-4/#typedef-pseudo-class-selector>

use crate::{
    css::{syntax::Token, CSSParse, ParseError, Parser},
    InternedString,
};

/// <https://drafts.csswg.org/selectors-4/#typedef-pseudo-class-selector>
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PseudoClassSelector {
    Ident(InternedString),
    Function,
}

impl<'a> CSSParse<'a> for PseudoClassSelector {
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        parser.expect_token(Token::Colon)?;

        let pseudo_class_selector = match parser.next_token_ignoring_whitespace() {
            Some(Token::Ident(ident)) => Self::Ident(ident),
            Some(Token::Function(_)) => Self::Function,
            _ => return Err(ParseError),
        };

        Ok(pseudo_class_selector)
    }
}
