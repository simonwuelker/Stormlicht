use std::fmt;

use crate::{
    css::{
        selectors::{CSSValidateSelector, Selector, Specificity, SubClassSelector, TypeSelector},
        syntax::WhitespaceAllowed,
        CSSParse, ParseError, Parser, Serialize, Serializer,
    },
    dom::{dom_objects::Element, DomPtr},
};

/// <https://drafts.csswg.org/selectors-4/#compound>
#[derive(Clone, Debug, PartialEq)]
pub struct CompoundSelector {
    pub type_selector: Option<TypeSelector>,
    pub subclass_selectors: Vec<SubClassSelector>,
}

/// <https://drafts.csswg.org/selectors-4/#typedef-compound-selector-list>
pub type CompoundSelectorList = Vec<CompoundSelector>;

impl<'a> CSSParse<'a> for CompoundSelector {
    // <https://drafts.csswg.org/selectors-4/#typedef-compound-selector>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        // Note that the selectors *must not* be seperated by whitespace
        let type_selector = parser.parse_optional_value(TypeSelector::parse);
        let subclass_selectors =
            parser.parse_any_number_of(SubClassSelector::parse, WhitespaceAllowed::No);

        Ok(CompoundSelector {
            type_selector,
            subclass_selectors,
        })
    }
}

impl<'a> CSSParse<'a> for CompoundSelectorList {
    // <https://drafts.csswg.org/selectors-4/#typedef-compound-selector-list>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        Ok(parser.parse_comma_seperated_list(CompoundSelector::parse))
    }
}

impl CSSValidateSelector for CompoundSelector {
    fn is_valid(&self) -> bool {
        if self.type_selector.as_ref().is_some_and(|t| !t.is_valid()) {
            return false;
        }
        self.subclass_selectors
            .iter()
            .all(CSSValidateSelector::is_valid)
    }
}

impl Selector for CompoundSelector {
    fn matches(&self, element: &DomPtr<Element>) -> bool {
        if self
            .type_selector
            .as_ref()
            .is_some_and(|s| !s.matches(element))
        {
            return false;
        }
        self.subclass_selectors.iter().all(|s| s.matches(element))
    }

    fn specificity(&self) -> Specificity {
        let mut specificity = Specificity::ZERO;

        if let Some(type_selector) = &self.type_selector {
            specificity += type_selector.specificity();
        }

        for subclass_selector in &self.subclass_selectors {
            specificity += subclass_selector.specificity();
        }

        specificity
    }
}

impl Serialize for CompoundSelector {
    fn serialize_to<T: Serializer>(&self, serializer: &mut T) -> fmt::Result {
        if let Some(type_selector) = &self.type_selector {
            type_selector.serialize_to(serializer)?;
        }

        for subclass_selector in &self.subclass_selectors {
            serializer.serialize(' ')?;
            subclass_selector.serialize_to(serializer)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::CompoundSelector;
    use crate::css::{CSSParse, ParseError};

    #[test]
    fn invalid_compound_selector() {
        // Spaces between selectors, invalid
        assert_eq!(
            CompoundSelector::parse_from_str("h1#foo bar"),
            Err(ParseError)
        );
    }
}
