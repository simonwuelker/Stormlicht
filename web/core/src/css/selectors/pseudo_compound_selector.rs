use super::{CSSValidateSelector, PseudoClassSelector, PseudoElementSelector};
use crate::css::{syntax::WhitespaceAllowed, CSSParse, ParseError, Parser};

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
        self.pseudo_element_selector.is_valid() && self.pseudo_class_selectors.is_valid()
    }
}
