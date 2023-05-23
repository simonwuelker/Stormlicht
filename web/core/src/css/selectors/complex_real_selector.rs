use super::{CSSValidateSelector, Combinator, CompoundSelector};
use crate::css::parser::{CSSParse, ParseError, Parser, WhitespaceAllowed};

/// <https://drafts.csswg.org/selectors-4/#typedef-complex-real-selector>
#[derive(Clone, Debug, PartialEq)]
pub struct ComplexRealSelector<'a> {
    pub first_selector: CompoundSelector<'a>,
    pub subsequent_selectors: Vec<(Combinator, CompoundSelector<'a>)>,
}

/// <https://drafts.csswg.org/selectors-4/#typedef-complex-real-selector-list>
pub type ComplexRealSelectorList<'a> = Vec<ComplexRealSelector<'a>>;

impl<'a> CSSParse<'a> for ComplexRealSelectorList<'a> {
    // <https://drafts.csswg.org/selectors-4/#typedef-complex-real-selector-list>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        Ok(parser.parse_comma_seperated_list(ComplexRealSelector::parse))
    }
}

impl<'a> CSSParse<'a> for ComplexRealSelector<'a> {
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

fn parse_selector_with_combinator<'a>(
    parser: &mut Parser<'a>,
) -> Result<(Combinator, CompoundSelector<'a>), ParseError> {
    parser.skip_whitespace();
    let combinator = parser
        .parse_optional_value(Combinator::parse)
        .unwrap_or_default();
    parser.skip_whitespace();
    let selector = CompoundSelector::parse(parser)?;

    Ok((combinator, selector))
}

impl<'a> CSSValidateSelector for ComplexRealSelector<'a> {
    fn is_valid(&self) -> bool {
        self.first_selector.is_valid()
            && self
                .subsequent_selectors
                .iter()
                .all(|(combinator, selector)| combinator.is_valid() && selector.is_valid())
    }
}
