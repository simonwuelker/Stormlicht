use std::fmt;

use crate::{
    css::{
        selectors::{CSSValidateSelector, Selector, Specificity},
        syntax::{HashFlag, Token},
        CSSParse, ParseError, Parser, Serialize, Serializer,
    },
    dom::{dom_objects::Element, DOMPtr},
    static_interned, InternedString,
};

/// <https://drafts.csswg.org/selectors-4/#id-selectors>
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

impl Serialize for IDSelector {
    fn serialize_to<T: Serializer>(&self, serializer: &mut T) -> fmt::Result {
        // Part of https://www.w3.org/TR/cssom-1/#serialize-a-simple-selector

        // Append a "#" (U+0023), followed by the serialization of the ID as an identifier to s.
        serializer.serialize('#')?;
        serializer.serialize_identifier(&self.ident.to_string())
    }
}
