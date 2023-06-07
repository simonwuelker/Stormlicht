use crate::css::{syntax::WhitespaceAllowed, CSSParse, ParseError, Parser};

use super::{CSSValidateSelector, Combinator, ComplexSelectorUnit};

/// <https://drafts.csswg.org/selectors-4/#typedef-complex-selector>
#[derive(Clone, Debug, PartialEq)]
pub struct ComplexSelector {
    pub first_unit: ComplexSelectorUnit,
    pub subsequent_units: Vec<(Combinator, ComplexSelectorUnit)>,
}

/// <https://drafts.csswg.org/selectors-4/#typedef-selector-list>
pub type SelectorList = ComplexSelectorList;

/// <https://drafts.csswg.org/selectors-4/#typedef-complex-selector-list>
pub type ComplexSelectorList = Vec<ComplexSelector>;

impl<'a> CSSParse<'a> for ComplexSelector {
    // <https://drafts.csswg.org/selectors-4/#typedef-complex-selector>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        let first_unit = ComplexSelectorUnit::parse(parser)?;
        let subsequent_units = parser.parse_any_number_of(
            parse_complex_selector_unit_with_combinator,
            WhitespaceAllowed::Yes,
        );

        Ok(ComplexSelector {
            first_unit,
            subsequent_units,
        })
    }
}

impl<'a> CSSParse<'a> for ComplexSelectorList {
    // <https://drafts.csswg.org/selectors-4/#typedef-complex-selector-list>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        Ok(parser.parse_comma_seperated_list(ComplexSelector::parse))
    }
}

fn parse_complex_selector_unit_with_combinator(
    parser: &mut Parser<'_>,
) -> Result<(Combinator, ComplexSelectorUnit), ParseError> {
    parser.skip_whitespace();
    let combinator = parser
        .parse_optional_value(Combinator::parse)
        .unwrap_or_default();
    parser.skip_whitespace();
    let complex_selector_unit = ComplexSelectorUnit::parse(parser)?;

    Ok((combinator, complex_selector_unit))
}

impl CSSValidateSelector for ComplexSelector {
    fn is_valid(&self) -> bool {
        self.first_unit.is_valid()
            && self
                .subsequent_units
                .iter()
                .all(|(combinator, unit)| combinator.is_valid() && unit.is_valid())
    }
}
