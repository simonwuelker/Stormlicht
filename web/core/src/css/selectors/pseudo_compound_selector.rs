use super::{CSSValidateSelector, PseudoClassSelector, PseudoElementSelector};
use crate::css::parser::{CSSParse, ParseError, Parser};

/// <https://drafts.csswg.org/selectors-4/#typedef-pseudo-compound-selector>
#[derive(Clone, Debug, PartialEq)]
pub struct PseudoCompoundSelector<'a> {
    pub pseudo_element_selector: PseudoElementSelector<'a>,
    pub pseudo_class_selectors: Vec<PseudoClassSelector<'a>>,
}

impl<'a> CSSParse<'a> for PseudoCompoundSelector<'a> {
    // <https://drafts.csswg.org/selectors-4/#typedef-pseudo-compound-selector>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        let pseudo_element_selector = PseudoElementSelector::parse(parser)?;

        let pseudo_class_selectors = parser.parse_any_number_of(PseudoClassSelector::parse);
        Ok(PseudoCompoundSelector {
            pseudo_element_selector,
            pseudo_class_selectors,
        })
    }
}

impl<'a> CSSValidateSelector for PseudoCompoundSelector<'a> {}
