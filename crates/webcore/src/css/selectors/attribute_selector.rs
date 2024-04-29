use std::fmt;

use crate::{
    css::{
        selectors::{
            AttributeMatcher, AttributeModifier, CSSValidateSelector, Specificity,
            WellQualifiedName,
        },
        syntax::Token,
        CSSParse, ParseError, Parser, Serialize, Serializer,
    },
    dom::{dom_objects::Element, DomPtr},
    InternedString,
};

/// <https://drafts.csswg.org/selectors-4/#attribute-selectors>
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AttributeSelector {
    Exists {
        attribute_name: WellQualifiedName,
    },
    Matches {
        attribute_name: WellQualifiedName,
        matcher: AttributeMatcher,
        value: String,
        modifier: AttributeModifier,
    },
}

impl AttributeSelector {
    /// Parse an attribute selector where the initial `[` has already been consumed
    pub fn parse_without_leading_bracket(parser: &mut Parser<'_>) -> Result<Self, ParseError> {
        let attribute_name = WellQualifiedName::parse(parser)?;

        let selector = if matches!(
            parser.peek_token_ignoring_whitespace(0),
            Some(Token::BracketClose)
        ) {
            let _ = parser.next_token_ignoring_whitespace();

            AttributeSelector::Exists { attribute_name }
        } else {
            let matcher = AttributeMatcher::parse(parser)?;
            let value = match parser.next_token_ignoring_whitespace() {
                Some(Token::String(value) | Token::Ident(value)) => value,
                _ => return Err(ParseError),
            };

            let modifier = parser
                .parse_optional_value(AttributeModifier::parse)
                .unwrap_or_default();

            parser.expect_token(Token::BracketClose)?;

            AttributeSelector::Matches {
                attribute_name,
                matcher,
                value: value.to_string(),
                modifier,
            }
        };

        Ok(selector)
    }

    pub fn matches(&self, element: &DomPtr<Element>) -> bool {
        match self {
            Self::Exists { attribute_name } => {
                // FIXME: Don't consider attribute namespace
                element
                    .borrow()
                    .attributes()
                    .get(&attribute_name.ident)
                    .is_some()
            },
            Self::Matches {
                attribute_name,
                matcher,
                value: selector_value,
                modifier,
            } => {
                let borrowed_elem = element.borrow();
                let attribute_value = borrowed_elem.attributes().get(&attribute_name.ident);

                match attribute_value {
                    Some(interned_value) => {
                        if modifier.is_case_insensitive() {
                            matcher.are_matching(
                                &selector_value.to_lowercase(),
                                &interned_value.to_string().to_ascii_lowercase(),
                            )
                        } else {
                            matcher.are_matching(
                                selector_value,
                                &interned_value.to_string().to_ascii_lowercase(),
                            )
                        }
                    },
                    None => false,
                }
            },
        }
    }
}

impl<'a> CSSParse<'a> for AttributeSelector {
    // <https://drafts.csswg.org/selectors-4/#typedef-attribute-selector>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        if !matches!(parser.next_token(), Some(Token::BracketOpen)) {
            return Err(ParseError);
        }

        Self::parse_without_leading_bracket(parser)
    }
}

impl CSSValidateSelector for AttributeSelector {
    fn is_valid(&self) -> bool {
        match self {
            Self::Exists { attribute_name } => attribute_name.is_valid(),
            Self::Matches {
                attribute_name,
                matcher,
                value: _,
                modifier,
            } => attribute_name.is_valid() && matcher.is_valid() && modifier.is_valid(),
        }
    }
}

impl Serialize for AttributeSelector {
    fn serialize_to<T: Serializer>(&self, serializer: &mut T) -> fmt::Result {
        serializer.serialize('[')?;

        match self {
            Self::Exists { attribute_name } => serializer.serialize(*attribute_name)?,
            Self::Matches {
                attribute_name,
                matcher,
                value,
                modifier,
            } => {
                serializer.serialize(*attribute_name)?;
                serializer.serialize(*matcher)?;
                serializer.serialize_identifier(&value.to_string())?;

                if modifier.is_case_insensitive() {
                    serializer.serialize(" i")?;
                }
            },
        }

        serializer.serialize(']')?;
        Ok(())
    }
}

impl AttributeMatcher {
    fn are_matching(&self, selector_value: &str, attribute_value: &str) -> bool {
        match self {
            Self::ContainsSubstring => attribute_value.contains(selector_value),
            Self::EndsWith => attribute_value.ends_with(selector_value),
            Self::EqualTo => attribute_value.eq(selector_value),
            Self::HyphenSeperatedListBeginningWith => {
                let following_char = attribute_value.as_bytes().get(selector_value.len() + 1);

                attribute_value.starts_with(selector_value)
                    && matches!(following_char, None | Some(b'-'))
            },
            Self::StartsWith => attribute_value.starts_with(selector_value),
            Self::WhiteSpaceSeperatedListContaining => attribute_value
                .split(|c: char| c.is_ascii_whitespace())
                .any(|element| element == selector_value),
        }
    }
}
