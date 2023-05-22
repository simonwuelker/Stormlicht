use super::{CSSValidateSelector, SubClassSelector, TypeSelector};
use crate::css::parser::{CSSParse, ParseError, Parser};

/// <https://drafts.csswg.org/selectors-4/#typedef-compound-selector>
#[derive(Clone, Debug, PartialEq)]
pub struct CompoundSelector<'a>(Vec<CompoundSelectorPart<'a>>);

#[derive(Clone, Debug, PartialEq)]
pub struct CompoundSelectorPart<'a> {
    pub type_selector: Option<TypeSelector<'a>>,
    pub subclass_selectors: Vec<SubClassSelector<'a>>,
}

/// <https://drafts.csswg.org/selectors-4/#typedef-compound-selector-list>
pub type CompoundSelectorList<'a> = Vec<CompoundSelector<'a>>;

impl<'a> CSSParse<'a> for CompoundSelector<'a> {
    // <https://drafts.csswg.org/selectors-4/#typedef-compound-selector>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        let compound_selector_parts = parser.parse_nonempty_comma_seperated_list(|parser| {
            let type_selector = parser.parse_optional_value(TypeSelector::parse);
            let subclass_selectors = parser.parse_any_number_of(SubClassSelector::parse);

            Ok(CompoundSelectorPart {
                type_selector,
                subclass_selectors,
            })
        })?;
        Ok(CompoundSelector(compound_selector_parts))
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
        self.0.iter().all(|p| p.is_valid())
    }
}

impl<'a> CSSValidateSelector for CompoundSelectorPart<'a> {
    fn is_valid(&self) -> bool {
        if self.type_selector.as_ref().is_some_and(|t| !t.is_valid()) {
            return false;
        }
        self.subclass_selectors.is_valid()
    }
}
