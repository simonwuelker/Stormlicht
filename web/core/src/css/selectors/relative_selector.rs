use super::{CSSValidateSelector, Combinator, ComplexSelector};
use crate::css::{CSSParse, ParseError, Parser};

/// <https://drafts.csswg.org/selectors-4/#typedef-relative-selector>
#[derive(Clone, Debug, PartialEq)]
pub struct RelativeSelector {
    pub combinator: Option<Combinator>,
    pub complex_selector: ComplexSelector,
}

/// <https://drafts.csswg.org/selectors-4/#typedef-relative-selector-list>
pub type RelativeSelectorList = Vec<RelativeSelector>;

impl<'a> CSSParse<'a> for RelativeSelector {
    // <https://drafts.csswg.org/selectors-4/#typedef-relative-selector>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        let combinator = parser.parse_optional_value(Combinator::parse);
        let complex_selector = ComplexSelector::parse(parser)?;

        Ok(RelativeSelector {
            combinator,
            complex_selector,
        })
    }
}

impl<'a> CSSParse<'a> for RelativeSelectorList {
    // <https://drafts.csswg.org/selectors-4/#typedef-relative-selector-list>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        Ok(parser.parse_comma_seperated_list(RelativeSelector::parse))
    }
}

impl CSSValidateSelector for RelativeSelector {
    fn is_valid(&self) -> bool {
        if self
            .combinator
            .is_some_and(|combinator| !combinator.is_valid())
        {
            return false;
        }

        self.complex_selector.is_valid()
    }
}
