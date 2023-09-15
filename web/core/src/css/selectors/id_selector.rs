use string_interner::InternedString;

use super::{CSSValidateSelector, Selector, Specificity};
use crate::{
    css::{
        syntax::{HashFlag, Token},
        CSSParse, ParseError, Parser,
    },
    dom::{dom_objects::Element, DOMPtr},
};

/// <https://drafts.csswg.org/selectors-4/#id-selectors>
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IDSelector {
    pub ident: InternedString,
}

impl<'a> CSSParse<'a> for IDSelector {
    // <https://drafts.csswg.org/selectors-4/#typedef-id-selector>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        if let Some(Token::Hash(ident, HashFlag::Id)) = parser.next_token() {
            Ok(IDSelector { ident })
        } else {
            Err(ParseError)
        }
    }
}

impl CSSValidateSelector for IDSelector {
    fn is_valid(&self) -> bool {
        true
    }
}

impl Selector for IDSelector {
    fn matches(&self, element: &DOMPtr<Element>) -> bool {
        element.borrow().id().is_some_and(|id| id == self.ident)
    }

    fn specificity(&self) -> Specificity {
        Specificity::new(1, 0, 0)
    }
}
