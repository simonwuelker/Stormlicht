use super::{CSSValidateSelector, Combinator, ComplexRealSelector};
use crate::css::{CSSParse, ParseError, Parser};

/// <https://drafts.csswg.org/selectors-4/#typedef-relative-real-selector>
#[derive(Clone, Debug, PartialEq)]
pub struct RelativeRealSelector {
    pub combinator: Option<Combinator>,
    pub complex_real_selector: ComplexRealSelector,
}

/// <https://drafts.csswg.org/selectors-4/#typedef-relative-real-selector-list>
pub type RelativeRealSelectorList = Vec<RelativeRealSelector>;

impl<'a> CSSParse<'a> for RelativeRealSelector {
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

impl<'a> CSSParse<'a> for RelativeRealSelectorList {
    // <https://drafts.csswg.org/selectors-4/#typedef-relative-real-selector-list>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        Ok(parser.parse_comma_seperated_list(RelativeRealSelector::parse))
    }
}

impl CSSValidateSelector for RelativeRealSelector {
    fn is_valid(&self) -> bool {
        if self
            .combinator
            .is_some_and(|combinator| !combinator.is_valid())
        {
            return false;
        }

        self.complex_real_selector.is_valid()
    }
}
