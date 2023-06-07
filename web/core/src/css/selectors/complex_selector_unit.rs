use super::{CSSValidateSelector, CompoundSelector, PseudoCompoundSelector};
use crate::css::{syntax::WhitespaceAllowed, CSSParse, ParseError, Parser};

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
        self.pseudo_compound_selectors.is_valid()
    }
}
