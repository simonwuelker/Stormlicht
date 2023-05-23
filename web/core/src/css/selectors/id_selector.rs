use std::borrow::Cow;

use super::{CSSValidateSelector, Selector};
use crate::{
    css::{
        syntax::{HashFlag, Token},
        CSSParse, ParseError, Parser,
    },
    dom::{dom_objects::Element, DOMPtr},
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

impl<'a> CSSValidateSelector for IDSelector<'a> {
    fn is_valid(&self) -> bool {
        true
    }
}

impl<'a> Selector for IDSelector<'a> {
    fn matches(&self, element: &DOMPtr<Element>) -> bool {
        self.ident == element.borrow().id()
    }
}
