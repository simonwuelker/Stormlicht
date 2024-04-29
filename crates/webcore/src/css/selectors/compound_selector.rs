use std::fmt;

use crate::{
    css::{
        selectors::{
            CSSValidateSelector, IDSelector, Selector, Specificity, SubClassSelector, TypeSelector,
        },
        syntax::{Token, WhitespaceAllowed},
        CSSParse, ParseError, Parser, Serialize, Serializer,
    },
    dom::{dom_objects::Element, DomPtr},
};

use super::{AttributeSelector, ClassSelector};

/// <https://drafts.csswg.org/selectors-4/#compound>
#[derive(Clone, Debug, PartialEq)]
pub struct CompoundSelector {
    pub type_selector: Option<TypeSelector>,
    pub subclass_selectors: Vec<SubClassSelector>,
}

/// <https://drafts.csswg.org/selectors-4/#typedef-compound-selector-list>
pub type CompoundSelectorList = Vec<CompoundSelector>;

impl CompoundSelector {
    /// Parse the remainder of a CompoundSelector
    /// where a type selector (optional) and some subclass selectors have already been consumed
    pub fn parse_subclass_selectors(
        parser: &mut Parser<'_>,
        type_selector: Option<TypeSelector>,
        mut subclass_selectors: Vec<SubClassSelector>,
    ) -> Result<Self, ParseError> {
        loop {
            // Whitespace is not allowed between the top components of a compound selector
            match parser.peek_token() {
                Some(Token::Hash(ident, ..)) => {
                    let id_selector = IDSelector { ident: *ident };
                    _ = parser.next_token();

                    subclass_selectors.push(id_selector.into())
                },
                Some(Token::Delim('.')) => {
                    // No whitespace allowed between the dot and the class name
                    let Some(Token::Ident(ident)) = parser.next_token() else {
                        return Err(ParseError);
                    };

                    let class_selector = ClassSelector { ident };
                    subclass_selectors.push(class_selector.into());
                },
                Some(Token::BracketOpen) => {
                    let _ = parser.next_token();
                    let attribute_selector =
                        AttributeSelector::parse_without_leading_bracket(parser)?;

                    subclass_selectors.push(attribute_selector.into())
                },

                _ => break,
            }
        }

        if type_selector.is_none() && subclass_selectors.is_empty() {
            return Err(ParseError);
        }

        let compound_selector = Self {
            type_selector,
            subclass_selectors,
        };

        Ok(compound_selector)
    }
}

impl<'a> CSSParse<'a> for CompoundSelector {
    // <https://drafts.csswg.org/selectors-4/#typedef-compound-selector>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        // Note that the selectors *must not* be seperated by whitespace
        let type_selector = parser.parse_optional_value(TypeSelector::parse);
        let subclass_selectors =
            parser.parse_any_number_of(SubClassSelector::parse, WhitespaceAllowed::No);

        Ok(CompoundSelector {
            type_selector,
            subclass_selectors,
        })
    }
}

impl<'a> CSSParse<'a> for CompoundSelectorList {
    // <https://drafts.csswg.org/selectors-4/#typedef-compound-selector-list>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        Ok(parser.parse_comma_seperated_list(CompoundSelector::parse))
    }
}

impl CSSValidateSelector for CompoundSelector {
    fn is_valid(&self) -> bool {
        if self.type_selector.as_ref().is_some_and(|t| !t.is_valid()) {
            return false;
        }
        self.subclass_selectors
            .iter()
            .all(CSSValidateSelector::is_valid)
    }
}

impl Selector for CompoundSelector {
    fn matches(&self, element: &DomPtr<Element>) -> bool {
        if self
            .type_selector
            .as_ref()
            .is_some_and(|s| !s.matches(element))
        {
            return false;
        }
        self.subclass_selectors.iter().all(|s| s.matches(element))
    }

    fn specificity(&self) -> Specificity {
        let mut specificity = Specificity::ZERO;

        if let Some(type_selector) = &self.type_selector {
            specificity += type_selector.specificity();
        }

        for subclass_selector in &self.subclass_selectors {
            specificity += subclass_selector.specificity();
        }

        specificity
    }
}

impl Serialize for CompoundSelector {
    fn serialize_to<T: Serializer>(&self, serializer: &mut T) -> fmt::Result {
        if let Some(type_selector) = &self.type_selector {
            type_selector.serialize_to(serializer)?;
        }

        for subclass_selector in &self.subclass_selectors {
            serializer.serialize(' ')?;
            subclass_selector.serialize_to(serializer)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::CompoundSelector;
    use crate::css::{CSSParse, ParseError};

    #[test]
    fn invalid_compound_selector() {
        // Spaces between selectors, invalid
        assert_eq!(
            CompoundSelector::parse_from_str("h1#foo bar"),
            Err(ParseError)
        );
    }
}
