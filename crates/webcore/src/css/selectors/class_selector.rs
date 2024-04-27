use std::fmt;

use crate::{
    css::{
        selectors::{CSSValidateSelector, Selector, Specificity},
        syntax::Token,
        CSSParse, ParseError, Parser, Serialize, Serializer,
    },
    dom::{dom_objects::Element, DomPtr},
    InternedString,
};

/// <https://drafts.csswg.org/selectors-4/#class-html>
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClassSelector {
    pub ident: InternedString,
}

impl<'a> CSSParse<'a> for ClassSelector {
    // <https://drafts.csswg.org/selectors-4/#typedef-class-selector>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        if let Some(Token::Delim('.')) = parser.next_token() {
            if let Some(Token::Ident(ident)) = parser.next_token() {
                return Ok(ClassSelector { ident });
            }
        }
        Err(ParseError)
    }
}

impl CSSValidateSelector for ClassSelector {
    fn is_valid(&self) -> bool {
        true
    }
}

impl Selector for ClassSelector {
    fn matches(&self, _element: &DomPtr<Element>) -> bool {
        false
    }

    fn specificity(&self) -> Specificity {
        Specificity::new(0, 1, 0)
    }
}

impl Serialize for ClassSelector {
    fn serialize_to<T: Serializer>(&self, serializer: &mut T) -> fmt::Result {
        // Part of https://www.w3.org/TR/cssom-1/#serialize-a-simple-selector

        // Append a "." (U+002E), followed by the serialization of the class name as an identifier to s.
        serializer.serialize('.')?;
        serializer.serialize_identifier(&self.ident.to_string())
    }
}
