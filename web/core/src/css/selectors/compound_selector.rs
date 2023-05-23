use super::{CSSValidateSelector, Selector, SubClassSelector, TypeSelector};
use crate::{
    css::parser::{CSSParse, ParseError, Parser, WhitespaceAllowed},
    dom::{dom_objects::Element, DOMPtr},
};
/// <https://drafts.csswg.org/selectors-4/#compound>
#[derive(Clone, Debug, PartialEq)]
pub struct CompoundSelector<'a> {
    pub type_selector: Option<TypeSelector<'a>>,
    pub subclass_selectors: Vec<SubClassSelector<'a>>,
}

/// <https://drafts.csswg.org/selectors-4/#typedef-compound-selector-list>
pub type CompoundSelectorList<'a> = Vec<CompoundSelector<'a>>;

impl<'a> CSSParse<'a> for CompoundSelector<'a> {
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

impl<'a> CSSParse<'a> for CompoundSelectorList<'a> {
    // <https://drafts.csswg.org/selectors-4/#typedef-compound-selector-list>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        Ok(parser.parse_comma_seperated_list(CompoundSelector::parse))
    }
}

impl<'a> CSSValidateSelector for CompoundSelector<'a> {
    fn is_valid(&self) -> bool {
        if self.type_selector.as_ref().is_some_and(|t| !t.is_valid()) {
            return false;
        }
        self.subclass_selectors.is_valid()
    }
}

impl<'a> Selector for CompoundSelector<'a> {
    fn matches(&self, element: &DOMPtr<Element>) -> bool {
        if self
            .type_selector
            .as_ref()
            .is_some_and(|s| !s.matches(element))
        {
            return false;
        }
        self.subclass_selectors.iter().all(|s| s.matches(element))
    }
}

#[cfg(test)]
mod tests {
    use super::CompoundSelector;
    use crate::css::{CSSParse, ParseError, Parser};

    #[test]
    fn invalid_compound_selector() {
        let mut spaces_between_selectors = Parser::new("h1#foo bar");
        assert_eq!(
            CompoundSelector::parse_complete(&mut spaces_between_selectors),
            Err(ParseError)
        );
    }
}
