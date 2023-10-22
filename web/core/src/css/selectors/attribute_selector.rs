use std::fmt;

use string_interner::InternedString;

use crate::{
    css::{syntax::Token, CSSParse, ParseError, Parser, Serialize, Serializer},
    dom::{dom_objects::Element, DOMPtr},
};

use super::{
    AttributeMatcher, AttributeModifier, CSSValidateSelector, Selector, Specificity, WQName,
};

/// <https://drafts.csswg.org/selectors-4/#attribute-selectors>
#[derive(Clone, Debug, PartialEq)]
pub enum AttributeSelector {
    Exists {
        attribute_name: WQName,
    },
    Matches {
        attribute_name: WQName,
        matcher: AttributeMatcher,
        value: String,
        modifier: AttributeModifier,
    },
}

impl<'a> CSSParse<'a> for AttributeSelector {
    // <https://drafts.csswg.org/selectors-4/#typedef-attribute-selector>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        if !matches!(parser.next_token(), Some(Token::BracketOpen)) {
            return Err(ParseError);
        }

        parser.skip_whitespace();

        // Both variants start with a wqname
        let attribute_name = WQName::parse(parser)?;

        parser.skip_whitespace();

        let value_matching_part = parser.parse_optional_value(parse_attribute_value_matcher);

        parser.skip_whitespace();

        if !matches!(parser.next_token(), Some(Token::BracketClose)) {
            return Err(ParseError);
        }

        match value_matching_part {
            Some((matcher, value, modifier)) => Ok(AttributeSelector::Matches {
                attribute_name,
                matcher,
                value: value.to_string(),
                modifier,
            }),
            None => Ok(AttributeSelector::Exists { attribute_name }),
        }
    }
}

fn parse_attribute_value_matcher(
    parser: &mut Parser<'_>,
) -> Result<(AttributeMatcher, InternedString, AttributeModifier), ParseError> {
    let attribute_matcher = AttributeMatcher::parse(parser)?;
    parser.skip_whitespace();
    let attribute_value = match parser.next_token() {
        Some(Token::String(value) | Token::Ident(value)) => value,
        _ => return Err(ParseError),
    };
    parser.skip_whitespace();
    let attribute_modifier = parser
        .parse_optional_value(AttributeModifier::parse)
        .unwrap_or_default();

    Ok((attribute_matcher, attribute_value, attribute_modifier))
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

impl Selector for AttributeSelector {
    fn matches(&self, element: &DOMPtr<Element>) -> bool {
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

    fn specificity(&self) -> Specificity {
        Specificity::new(0, 1, 0)
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
#[cfg(test)]
mod tests {
    use super::AttributeSelector;
    use crate::css::{
        selectors::{
            AttributeMatcher, AttributeModifier, ClassSelector, IDSelector, PseudoClassSelector,
            SubClassSelector, WQName,
        },
        CSSParse,
    };

    #[test]
    fn parse_subclass_selector() {
        assert_eq!(
            SubClassSelector::parse_from_str("#foo"),
            Ok(SubClassSelector::ID(IDSelector {
                ident: "foo".into()
            }))
        );

        assert_eq!(
            SubClassSelector::parse_from_str(".foo"),
            Ok(SubClassSelector::Class(ClassSelector {
                ident: "foo".into()
            }))
        );

        assert_eq!(
            SubClassSelector::parse_from_str("[foo]"),
            Ok(SubClassSelector::Attribute(AttributeSelector::Exists {
                attribute_name: WQName {
                    prefix: None,
                    ident: "foo".into(),
                }
            }))
        );

        assert_eq!(
            SubClassSelector::parse_from_str(":foo"),
            Ok(SubClassSelector::PseudoClass(PseudoClassSelector::Ident(
                "foo".into()
            )))
        );
    }

    #[test]
    fn parse_id_selector() {
        assert_eq!(
            IDSelector::parse_from_str("#foo"),
            Ok(IDSelector {
                ident: "foo".into()
            })
        )
    }

    #[test]
    fn parse_class_selector() {
        assert_eq!(
            ClassSelector::parse_from_str(".foo"),
            Ok(ClassSelector {
                ident: "foo".into()
            })
        )
    }

    #[test]
    fn parse_attribute_selector() {
        assert_eq!(
            AttributeSelector::parse_from_str("[foo]"),
            Ok(AttributeSelector::Exists {
                attribute_name: WQName {
                    prefix: None,
                    ident: "foo".into(),
                }
            })
        );

        assert_eq!(
            AttributeSelector::parse_from_str("[foo ^= bar i]"),
            Ok(AttributeSelector::Matches {
                attribute_name: WQName {
                    prefix: None,
                    ident: "foo".into(),
                },
                matcher: AttributeMatcher::StartsWith,
                value: "bar".into(),
                modifier: AttributeModifier::CaseInsensitive
            })
        );

        assert_eq!(
            AttributeSelector::parse_from_str("[foo $= bar]"),
            Ok(AttributeSelector::Matches {
                attribute_name: WQName {
                    prefix: None,
                    ident: "foo".into(),
                },
                matcher: AttributeMatcher::EndsWith,
                value: "bar".into(),
                modifier: AttributeModifier::CaseSensitive
            })
        );
    }
}
