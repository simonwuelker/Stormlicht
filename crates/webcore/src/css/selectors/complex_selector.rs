use std::fmt;

use crate::{
    css::{
        selectors::{CSSValidateSelector, Combinator, ComplexSelectorUnit, Selector, Specificity},
        syntax::{Token, WhitespaceAllowed},
        CSSParse, ParseError, Parser, Serialize, Serializer,
    },
    dom::{dom_objects::Element, DomPtr},
};

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

        let mut subsequent_units = vec![];
        loop {
            let combinator = match parser.peek_token_ignoring_whitespace(0) {
                None => {
                    // There are no more units
                    break;
                },
                Some(Token::Delim('>')) => {
                    _ = parser.next_token_ignoring_whitespace();
                    Combinator::Child
                },
                Some(Token::Delim('+')) => {
                    _ = parser.next_token_ignoring_whitespace();
                    Combinator::NextSibling
                },
                Some(Token::Delim('~')) => {
                    _ = parser.next_token_ignoring_whitespace();
                    Combinator::SubsequentSibling
                },
                Some(Token::Delim('|')) => {
                    _ = parser.next_token_ignoring_whitespace();
                    if !matches!(parser.next_token(), Some(Token::Delim('|'))) {
                        return Err(ParseError);
                    }

                    Combinator::Column
                },
                Some(_) => {
                    // There is no combinator between these two complex selector units.
                    // There *must* be at least one whitespace (descendant combinator)
                    if !parser
                        .next_token()
                        .as_ref()
                        .is_some_and(Token::is_whitespace)
                    {
                        return Err(ParseError);
                    }

                    Combinator::Descendant
                },
            };

            let next_unit = ComplexSelectorUnit::parse(parser)?;

            subsequent_units.push((combinator, next_unit));
        }

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

impl CSSValidateSelector for ComplexSelector {
    fn is_valid(&self) -> bool {
        self.first_unit.is_valid()
            && self
                .subsequent_units
                .iter()
                .all(|(combinator, unit)| combinator.is_valid() && unit.is_valid())
    }
}

impl Selector for ComplexSelector {
    fn matches(&self, element: &DomPtr<Element>) -> bool {
        // https://drafts.csswg.org/selectors-4/#match-a-complex-selector-against-an-element
        // FIXME:
        // Implement combinators, we only match on the rightmost compound selector
        for (_combinator, selector_unit) in self.subsequent_units.iter().rev() {
            if let Some(compound_selector) = &selector_unit.compound_selector {
                return compound_selector.matches(element);
            }
        }
        if let Some(compound_selector) = &self.first_unit.compound_selector {
            compound_selector.matches(element)
        } else {
            false
        }
    }

    fn specificity(&self) -> Specificity {
        self.first_unit.specificity()
            + self
                .subsequent_units
                .iter()
                .map(|(_combinator, unit)| unit)
                .map(Selector::specificity)
                .sum()
    }
}

impl Serialize for &ComplexSelector {
    fn serialize_to<T: Serializer>(&self, serializer: &mut T) -> fmt::Result {
        self.first_unit.serialize_to(serializer)?;

        for (combinator, subsequent_unit) in &self.subsequent_units {
            serializer.serialize(' ')?;
            combinator.serialize_to(serializer)?;

            if !combinator.is_descendant() {
                serializer.serialize(' ')?;
            }

            subsequent_unit.serialize_to(serializer)?;
        }
        Ok(())
    }
}
