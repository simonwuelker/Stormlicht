use std::fmt;

use super::{
    AttributeSelector, CSSValidateSelector, ClassSelector, IDSelector, PseudoClassSelector,
    Selector,
};
use crate::{
    css::{CSSParse, ParseError, Parser, Serialize, Serializer},
    dom::{dom_objects::Element, DOMPtr},
};

/// <https://drafts.csswg.org/selectors-4/#typedef-subclass-selector>
#[derive(Clone, Debug, PartialEq)]
pub enum SubClassSelector {
    ID(IDSelector),
    Class(ClassSelector),
    Attribute(AttributeSelector),
    PseudoClass(PseudoClassSelector),
}

impl<'a> CSSParse<'a> for SubClassSelector {
    // <https://drafts.csswg.org/selectors-4/#typedef-subclass-selector>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        let start_state = parser.state();
        if let Ok(id_selector) = IDSelector::parse(parser) {
            return Ok(SubClassSelector::ID(id_selector));
        }

        parser.set_state(start_state.clone());
        if let Ok(class_selector) = ClassSelector::parse(parser) {
            return Ok(SubClassSelector::Class(class_selector));
        }

        parser.set_state(start_state.clone());
        if let Ok(attribute_selector) = AttributeSelector::parse(parser) {
            return Ok(SubClassSelector::Attribute(attribute_selector));
        }

        parser.set_state(start_state);
        if let Ok(pseudoclass_selector) = PseudoClassSelector::parse(parser) {
            return Ok(SubClassSelector::PseudoClass(pseudoclass_selector));
        }

        Err(ParseError)
    }
}

impl CSSValidateSelector for SubClassSelector {
    fn is_valid(&self) -> bool {
        match self {
            Self::ID(id_selector) => id_selector.is_valid(),
            Self::Class(class_selector) => class_selector.is_valid(),
            Self::Attribute(attribute_selector) => attribute_selector.is_valid(),
            Self::PseudoClass(pseudo_class_selector) => pseudo_class_selector.is_valid(),
        }
    }
}

impl Selector for SubClassSelector {
    fn matches(&self, element: &DOMPtr<Element>) -> bool {
        match self {
            Self::ID(id_selector) => id_selector.matches(element),
            Self::Class(class_selector) => class_selector.matches(element),
            Self::Attribute(attribute_selector) => attribute_selector.matches(element),
            Self::PseudoClass(pseudo_class_selector) => pseudo_class_selector.matches(element),
        }
    }

    fn specificity(&self) -> super::Specificity {
        match self {
            Self::ID(id_selector) => id_selector.specificity(),
            Self::Class(class_selector) => class_selector.specificity(),
            Self::Attribute(attribute_selector) => attribute_selector.specificity(),
            Self::PseudoClass(pseudo_class_selector) => pseudo_class_selector.specificity(),
        }
    }
}

impl Serialize for &SubClassSelector {
    fn serialize_to<T: Serializer>(&self, serializer: &mut T) -> fmt::Result {
        match self {
            SubClassSelector::ID(id_selector) => id_selector.serialize_to(serializer),
            SubClassSelector::Class(class_selector) => class_selector.serialize_to(serializer),
            SubClassSelector::Attribute(attribute_selector) => {
                attribute_selector.serialize_to(serializer)
            },
            SubClassSelector::PseudoClass(pseudo_class_selector) => {
                pseudo_class_selector.serialize_to(serializer)
            },
        }
    }
}
