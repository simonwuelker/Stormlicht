use super::{CSSValidateSelector, CompoundSelector, PseudoCompoundSelector};
use crate::css::{syntax::WhitespaceAllowed, CSSParse, ParseError, Parser};

/// <https://drafts.csswg.org/selectors-4/#typedef-complex-selector-unit>
#[derive(Clone, Debug, PartialEq)]
pub struct ComplexSelectorUnit<'a>(pub Vec<ComplexSelectorUnitPart<'a>>);

#[derive(Clone, Debug, PartialEq)]
pub struct ComplexSelectorUnitPart<'a> {
    pub compound_selector: Option<CompoundSelector<'a>>,
    pub pseudo_compound_selectors: Vec<PseudoCompoundSelector<'a>>,
}

impl<'a> CSSParse<'a> for ComplexSelectorUnit<'a> {
    // <https://drafts.csswg.org/selectors-4/#typedef-complex-selector-unit>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        let complex_selector_unit_parts =
            parser.parse_nonempty_comma_seperated_list(parse_complex_selector_unit_part)?;
        Ok(ComplexSelectorUnit(complex_selector_unit_parts))
    }
}

fn parse_complex_selector_unit_part<'a>(
    parser: &mut Parser<'a>,
) -> Result<ComplexSelectorUnitPart<'a>, ParseError> {
    let compound_selector = parser.parse_optional_value(CompoundSelector::parse);
    let pseudo_compound_selectors =
        parser.parse_any_number_of(PseudoCompoundSelector::parse, WhitespaceAllowed::Yes);
    Ok(ComplexSelectorUnitPart {
        compound_selector,
        pseudo_compound_selectors,
    })
}

impl<'a> CSSValidateSelector for ComplexSelectorUnit<'a> {
    fn is_valid(&self) -> bool {
        self.0.is_valid()
    }
}

impl<'a> CSSValidateSelector for ComplexSelectorUnitPart<'a> {
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
