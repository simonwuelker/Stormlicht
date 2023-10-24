use std::fmt;

use crate::{
    css::{
        selectors::{CSSValidateSelector, Selector, SubClassSelector, TypeSelector},
        CSSParse, ParseError, Parser, Serialize, Serializer,
    },
    dom::{dom_objects::Element, DOMPtr},
};

/// <https://drafts.csswg.org/selectors-4/#simple>
#[derive(Clone, Debug, PartialEq)]
pub enum SimpleSelector {
    Type(TypeSelector),
    SubClass(SubClassSelector),
}

/// <https://drafts.csswg.org/selectors-4/#typedef-simple-selector-list>
pub type SimpleSelectorList = Vec<SimpleSelector>;

impl<'a> CSSParse<'a> for SimpleSelector {
    // <https://drafts.csswg.org/selectors-4/#typedef-simple-selector>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        let start_state = parser.state();
        if let Ok(type_selector) = TypeSelector::parse(parser) {
            return Ok(SimpleSelector::Type(type_selector));
        }

        parser.set_state(start_state);
        if let Ok(subclass_selector) = SubClassSelector::parse(parser) {
            return Ok(SimpleSelector::SubClass(subclass_selector));
        }

        Err(ParseError)
    }
}

impl<'a> CSSParse<'a> for SimpleSelectorList {
    // <https://drafts.csswg.org/selectors-4/#typedef-simple-selector-list>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        Ok(parser.parse_comma_seperated_list(SimpleSelector::parse))
    }
}

impl CSSValidateSelector for SimpleSelector {
    fn is_valid(&self) -> bool {
        match self {
            Self::Type(type_selector) => type_selector.is_valid(),
            Self::SubClass(subclass_selector) => subclass_selector.is_valid(),
        }
    }
}

impl Selector for SimpleSelector {
    fn matches(&self, element: &DOMPtr<Element>) -> bool {
        match self {
            Self::Type(type_selector) => type_selector.matches(element),
            Self::SubClass(subclass_selector) => subclass_selector.matches(element),
        }
    }

    fn specificity(&self) -> super::Specificity {
        match self {
            Self::Type(type_selector) => type_selector.specificity(),
            Self::SubClass(subclass_selector) => subclass_selector.specificity(),
        }
    }
}

impl Serialize for SimpleSelector {
    // https://www.w3.org/TR/cssom-1/#serialize-a-simple-selector
    fn serialize_to<T: Serializer>(&self, serializer: &mut T) -> fmt::Result {
        match self {
            Self::Type(type_selector) => type_selector.serialize_to(serializer),
            Self::SubClass(subclass_selector) => subclass_selector.serialize_to(serializer),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SimpleSelector;
    use crate::css::{
        selectors::{IDSelector, SubClassSelector, TypeSelector, WQName},
        CSSParse,
    };

    #[test]
    fn parse_simple_selector() {
        assert_eq!(
            SimpleSelector::parse_from_str("foo"),
            Ok(SimpleSelector::Type(TypeSelector::WQName(WQName {
                prefix: None,
                ident: "foo".into()
            })))
        );

        assert_eq!(
            SimpleSelector::parse_from_str("#foo"),
            Ok(SimpleSelector::SubClass(SubClassSelector::ID(IDSelector {
                ident: "foo".into()
            })))
        );
    }
}
