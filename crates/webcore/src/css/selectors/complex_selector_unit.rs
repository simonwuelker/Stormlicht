use std::fmt;

use crate::{
    css::{
        selectors::{
            CSSValidateSelector, ClassSelector, CompoundSelector, IDSelector,
            PseudoCompoundSelector, Selector, Specificity, TypeSelector, WellQualifiedName,
        },
        syntax::{Token, WhitespaceAllowed},
        CSSParse, ParseError, Parser, Serialize, Serializer,
    },
    dom::{dom_objects::Element, DomPtr},
};

use super::AttributeSelector;

/// <https://drafts.csswg.org/selectors-4/#typedef-complex-selector-unit>
#[derive(Clone, Debug, PartialEq)]
pub struct ComplexSelectorUnit {
    pub compound_selector: Option<CompoundSelector>,
    pub pseudo_compound_selectors: Vec<PseudoCompoundSelector>,
}

impl ComplexSelectorUnit {
    fn parse_with_compound_selector(
        parser: &mut Parser<'_>,
        compound_selector: CompoundSelector,
    ) -> Result<Self, ParseError> {
        // Any number (possibly zero) pseudo compound selectors follow
        // No whitespace between them is allowed
        let mut pseudo_compound_selectors = vec![];
        loop {
            match parser.peek_token() {
                Some(Token::Delim(':')) => {
                    let pseudo_compound_selector = PseudoCompoundSelector::parse(parser)?;
                    pseudo_compound_selectors.push(pseudo_compound_selector);
                },
                _ => {
                    // No more pseudo compound selectors
                    break;
                },
            }
        }

        let complex_selector_unit = Self {
            compound_selector: Some(compound_selector),
            pseudo_compound_selectors,
        };

        Ok(complex_selector_unit)
    }
}

impl<'a> CSSParse<'a> for ComplexSelectorUnit {
    // <https://drafts.csswg.org/selectors-4/#typedef-complex-selector-unit>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        parser.parse_nonempty(|parser| {
            let complex_selector_unit = match parser.next_token_ignoring_whitespace() {
                Some(Token::Ident(ident)) => {
                    // Compound selector -> type selector
                    let type_name = WellQualifiedName::without_namespace(ident);
                    let type_selector = TypeSelector::Typename(type_name);

                    let compound_selector = CompoundSelector::parse_subclass_selectors(
                        parser,
                        Some(type_selector),
                        vec![],
                    )?;

                    Self::parse_with_compound_selector(parser, compound_selector)?
                },
                Some(Token::Hash(ident, ..)) => {
                    // ID selector
                    let id_selector = IDSelector { ident };

                    let compound_selector = CompoundSelector::parse_subclass_selectors(
                        parser,
                        None,
                        vec![id_selector.into()],
                    )?;

                    Self::parse_with_compound_selector(parser, compound_selector)?
                },
                Some(Token::Delim('.')) => {
                    // No whitespace is allowed between the dot and the identifier
                    let Some(Token::Ident(ident)) = parser.next_token() else {
                        return Err(ParseError);
                    };

                    let class_selector = ClassSelector { ident };

                    let compound_selector = CompoundSelector::parse_subclass_selectors(
                        parser,
                        None,
                        vec![class_selector.into()],
                    )?;

                    Self::parse_with_compound_selector(parser, compound_selector)?
                },
                Some(Token::BracketOpen) => {
                    // This is either the namespace prefix of a type selector
                    // or the start of an attribute selector.
                    if matches!(
                        parser.peek_token_ignoring_whitespace(3),
                        Some(Token::Delim('|'))
                    ) {
                        // Namespace prefix
                        todo!()
                    } else {
                        // Attribute selector
                        let attribute_selector =
                            AttributeSelector::parse_without_leading_bracket(parser)?;

                        let compound_selector = CompoundSelector::parse_subclass_selectors(
                            parser,
                            None,
                            vec![attribute_selector.into()],
                        )?;

                        Self::parse_with_compound_selector(parser, compound_selector)?
                    }
                },
                _ => todo!(),
            };

            Ok(complex_selector_unit)
        })
    }
}

impl CSSValidateSelector for ComplexSelectorUnit {
    fn is_valid(&self) -> bool {
        // We don't care if there's no compound selector
        if self
            .compound_selector
            .as_ref()
            .is_some_and(|c| !c.is_valid())
        {
            return false;
        }
        self.pseudo_compound_selectors
            .iter()
            .all(CSSValidateSelector::is_valid)
    }
}

impl Selector for ComplexSelectorUnit {
    fn matches(&self, element: &DomPtr<Element>) -> bool {
        !self
            .compound_selector
            .as_ref()
            .is_some_and(|selector| !selector.matches(element))
            && self
                .pseudo_compound_selectors
                .iter()
                .all(|selector| selector.matches(element))
    }

    fn specificity(&self) -> Specificity {
        self.compound_selector
            .as_ref()
            .map(Selector::specificity)
            .unwrap_or(Specificity::ZERO)
            + self
                .pseudo_compound_selectors
                .iter()
                .map(Selector::specificity)
                .sum()
    }
}

impl Serialize for ComplexSelectorUnit {
    fn serialize_to<T: Serializer>(&self, serializer: &mut T) -> fmt::Result {
        if let Some(compound_selector) = &self.compound_selector {
            compound_selector.serialize_to(serializer)?;
        }

        for pseudo_selector in &self.pseudo_compound_selectors {
            serializer.serialize(' ')?;

            // FIXME: Serializer pseudo compound selectors
            _ = pseudo_selector;
        }

        Ok(())
    }
}
