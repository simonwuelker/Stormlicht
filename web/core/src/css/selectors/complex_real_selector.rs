use super::{CSSValidateSelector, Combinator, CompoundSelector};
use crate::css::{syntax::WhitespaceAllowed, CSSParse, ParseError, Parser};

/// <https://drafts.csswg.org/selectors-4/#typedef-complex-real-selector>
#[derive(Clone, Debug, PartialEq)]
pub struct ComplexRealSelector {
    pub first_selector: CompoundSelector,
    pub subsequent_selectors: Vec<(Combinator, CompoundSelector)>,
}

/// <https://drafts.csswg.org/selectors-4/#typedef-complex-real-selector-list>
pub type ComplexRealSelectorList = Vec<ComplexRealSelector>;

impl<'a> CSSParse<'a> for ComplexRealSelectorList {
    // <https://drafts.csswg.org/selectors-4/#typedef-complex-real-selector-list>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        Ok(parser.parse_comma_seperated_list(ComplexRealSelector::parse))
    }
}

impl<'a> CSSParse<'a> for ComplexRealSelector {
    // <https://drafts.csswg.org/selectors-4/#typedef-complex-real-selector>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        let first_selector = CompoundSelector::parse(parser)?;

        let subsequent_selectors =
            parser.parse_any_number_of(parse_selector_with_combinator, WhitespaceAllowed::Yes);

        Ok(ComplexRealSelector {
            first_selector,
            subsequent_selectors,
        })
    }
}

fn parse_selector_with_combinator(
    parser: &mut Parser<'_>,
) -> Result<(Combinator, CompoundSelector), ParseError> {
    parser.skip_whitespace();
    let combinator = parser
        .parse_optional_value(Combinator::parse)
        .unwrap_or_default();
    parser.skip_whitespace();
    let selector = CompoundSelector::parse(parser)?;

    Ok((combinator, selector))
}

impl CSSValidateSelector for ComplexRealSelector {
    fn is_valid(&self) -> bool {
        self.first_selector.is_valid()
            && self
                .subsequent_selectors
                .iter()
                .all(|(combinator, selector)| combinator.is_valid() && selector.is_valid())
    }
}
