use std::fmt;

use super::{
    CSSValidateSelector, PseudoClassSelector, PseudoElementSelector, Selector, Specificity,
};
use crate::{
    css::{syntax::WhitespaceAllowed, CSSParse, ParseError, Parser, Serialize, Serializer},
    dom::{dom_objects::Element, DOMPtr},
};

/// <https://drafts.csswg.org/selectors-4/#typedef-pseudo-compound-selector>
#[derive(Clone, Debug, PartialEq)]
pub struct PseudoCompoundSelector {
    pub pseudo_element_selector: PseudoElementSelector,
    pub pseudo_class_selectors: Vec<PseudoClassSelector>,
}

impl<'a> CSSParse<'a> for PseudoCompoundSelector {
    // <https://drafts.csswg.org/selectors-4/#typedef-pseudo-compound-selector>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        let pseudo_element_selector = PseudoElementSelector::parse(parser)?;

        let pseudo_class_selectors =
            parser.parse_any_number_of(PseudoClassSelector::parse, WhitespaceAllowed::Yes);
        Ok(PseudoCompoundSelector {
            pseudo_element_selector,
            pseudo_class_selectors,
        })
    }
}

impl CSSValidateSelector for PseudoCompoundSelector {
    fn is_valid(&self) -> bool {
        self.pseudo_element_selector.is_valid()
            && self
                .pseudo_class_selectors
                .iter()
                .all(CSSValidateSelector::is_valid)
    }
}

impl Selector for PseudoCompoundSelector {
    fn matches(&self, element: &DOMPtr<Element>) -> bool {
        self.pseudo_element_selector.matches(element)
            && self
                .pseudo_class_selectors
                .iter()
                .all(|selector| selector.matches(element))
    }

    fn specificity(&self) -> Specificity {
        self.pseudo_element_selector.specificity()
            + self
                .pseudo_class_selectors
                .iter()
                .map(Selector::specificity)
                .sum()
    }
}

impl Serialize for PseudoCompoundSelector {
    fn serialize_to<T: Serializer>(&self, serializer: &mut T) -> fmt::Result {
        self.pseudo_element_selector.serialize_to(serializer)?;

        for pseudo_class_selector in &self.pseudo_class_selectors {
            serializer.serialize(' ')?;
            pseudo_class_selector.serialize_to(serializer)?;
        }

        Ok(())
    }
}
