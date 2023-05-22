use std::borrow::Cow;

use super::CSSValidateSelector;
use crate::css::{
    parser::{CSSParse, ParseError, Parser},
    tokenizer::{HashFlag, Token},
};

/// <https://drafts.csswg.org/selectors-4/#typedef-id-selector>
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IDSelector<'a> {
    pub ident: Cow<'a, str>,
}

impl<'a> CSSParse<'a> for IDSelector<'a> {
    // <https://drafts.csswg.org/selectors-4/#typedef-id-selector>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        if let Some(Token::Hash(ident, HashFlag::Id)) = parser.next_token() {
            Ok(IDSelector { ident })
        } else {
            Err(ParseError)
        }
    }
}

impl<'a> CSSValidateSelector for IDSelector<'a> {}
