use crate::css::parser::{CSSParse, ParseError, Parser};

use super::{CSSValidateSelector, Combinator, ComplexSelectorUnit};

/// <https://drafts.csswg.org/selectors-4/#typedef-complex-selector>
#[derive(Clone, Debug, PartialEq)]
pub struct ComplexSelector<'a> {
    pub first_unit: ComplexSelectorUnit<'a>,
    pub subsequent_units: Vec<(Combinator, ComplexSelectorUnit<'a>)>,
}

/// <https://drafts.csswg.org/selectors-4/#typedef-selector-list>
pub type SelectorList<'a> = ComplexSelectorList<'a>;

/// <https://drafts.csswg.org/selectors-4/#typedef-complex-selector-list>
pub type ComplexSelectorList<'a> = Vec<ComplexSelector<'a>>;

impl<'a> CSSParse<'a> for ComplexSelector<'a> {
    // <https://drafts.csswg.org/selectors-4/#typedef-complex-selector>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        let first_unit = ComplexSelectorUnit::parse(parser)?;
        let subsequent_units =
            parser.parse_any_number_of(parse_complex_selector_unit_with_combinator);

        Ok(ComplexSelector {
            first_unit,
            subsequent_units,
        })
    }
}

impl<'a> CSSParse<'a> for ComplexSelectorList<'a> {
    // <https://drafts.csswg.org/selectors-4/#typedef-complex-selector-list>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        Ok(parser.parse_comma_seperated_list(ComplexSelector::parse))
    }
}

fn parse_complex_selector_unit_with_combinator<'a>(
    parser: &mut Parser<'a>,
) -> Result<(Combinator, ComplexSelectorUnit<'a>), ParseError> {
    parser.skip_whitespace();
    let combinator = parser
        .parse_optional_value(Combinator::parse)
        .unwrap_or_default();
    parser.skip_whitespace();
    let complex_selector_unit = ComplexSelectorUnit::parse(parser)?;

    Ok((combinator, complex_selector_unit))
}

impl<'a> CSSValidateSelector for ComplexSelector<'a> {
    fn is_valid(&self) -> bool {
        self.first_unit.is_valid() && self.subsequent_units.iter().all(|unit| unit.1.is_valid())
    }
}

impl<'a> CSSValidateSelector for ComplexSelectorList<'a> {
    fn is_valid(&self) -> bool {
        self.iter().all(|element| element.is_valid())
    }
}
