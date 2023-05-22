use super::{CSSValidateSelector, Combinator, ComplexRealSelector};
use crate::css::parser::{CSSParse, ParseError, Parser};

/// <https://drafts.csswg.org/selectors-4/#typedef-relative-real-selector>
#[derive(Clone, Debug, PartialEq)]
pub struct RelativeRealSelector<'a> {
    pub combinator: Option<Combinator>,
    pub complex_real_selector: ComplexRealSelector<'a>,
}

/// <https://drafts.csswg.org/selectors-4/#typedef-relative-real-selector-list>
pub type RelativeRealSelectorList<'a> = Vec<RelativeRealSelector<'a>>;

impl<'a> CSSParse<'a> for RelativeRealSelector<'a> {
    // <https://drafts.csswg.org/selectors-4/#typedef-relative-real-selector>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        let combinator = parser.parse_optional_value(Combinator::parse);
        let complex_real_selector = ComplexRealSelector::parse(parser)?;

        Ok(RelativeRealSelector {
            combinator,
            complex_real_selector,
        })
    }
}

impl<'a> CSSParse<'a> for RelativeRealSelectorList<'a> {
    // <https://drafts.csswg.org/selectors-4/#typedef-relative-real-selector-list>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        Ok(parser.parse_comma_seperated_list(RelativeRealSelector::parse))
    }
}

impl<'a> CSSValidateSelector for RelativeRealSelector<'a> {}
