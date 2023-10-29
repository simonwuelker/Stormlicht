use std::fmt;

use crate::{
    css::{
        selectors::{
            CSSValidateSelector, CompoundSelector, PseudoCompoundSelector, Selector, Specificity,
        },
        syntax::WhitespaceAllowed,
        CSSParse, ParseError, Parser, Serialize, Serializer,
    },
    dom::{dom_objects::Element, DOMPtr},
};

/// <https://drafts.csswg.org/selectors-4/#typedef-complex-selector-unit>
#[derive(Clone, Debug, PartialEq)]
pub struct ComplexSelectorUnit {
    pub compound_selector: Option<CompoundSelector>,
    pub pseudo_compound_selectors: Vec<PseudoCompoundSelector>,
}

impl<'a> CSSParse<'a> for ComplexSelectorUnit {
    // <https://drafts.csswg.org/selectors-4/#typedef-complex-selector-unit>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        parser.parse_nonempty(|parser| {
            let compound_selector = parser.parse_optional_value(CompoundSelector::parse);
            let pseudo_compound_selectors =
                parser.parse_any_number_of(PseudoCompoundSelector::parse, WhitespaceAllowed::Yes);

            Ok(ComplexSelectorUnit {
                compound_selector,
                pseudo_compound_selectors,
            })
        })
    }
}

impl CSSValidateSelector for ComplexSelectorUnit {
    fn is_valid(&self) -> bool {
        // We don't care if there's no compound selector
        if self
            .compound_selector
            .as_ref()
            .is_some_and(|c| !c.is_valid())
        {
            return false;
        }
        self.pseudo_compound_selectors
            .iter()
            .all(CSSValidateSelector::is_valid)
    }
}

impl Selector for ComplexSelectorUnit {
    fn matches(&self, element: &DOMPtr<Element>) -> bool {
        !self
            .compound_selector
            .as_ref()
            .is_some_and(|selector| !selector.matches(element))
            && self
                .pseudo_compound_selectors
                .iter()
                .all(|selector| selector.matches(element))
    }

    fn specificity(&self) -> Specificity {
        self.compound_selector
            .as_ref()
            .map(Selector::specificity)
            .unwrap_or(Specificity::ZERO)
            + self
                .pseudo_compound_selectors
                .iter()
                .map(Selector::specificity)
                .sum()
    }
}

impl Serialize for ComplexSelectorUnit {
    fn serialize_to<T: Serializer>(&self, serializer: &mut T) -> fmt::Result {
        if let Some(compound_selector) = &self.compound_selector {
            compound_selector.serialize_to(serializer)?;
        }

        for pseudo_selector in &self.pseudo_compound_selectors {
            serializer.serialize(' ')?;

            // FIXME: Serializer pseudo compound selectors
            _ = pseudo_selector;
        }

        Ok(())
    }
}
