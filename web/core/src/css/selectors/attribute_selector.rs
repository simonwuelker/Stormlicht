use std::borrow::Cow;

use crate::css::{syntax::Token, CSSParse, ParseError, Parser};

use super::{AttributeMatcher, AttributeModifier, CSSValidateSelector, WQName};

/// <https://drafts.csswg.org/selectors-4/#typedef-attribute-selector>
#[derive(Clone, Debug, PartialEq)]
pub enum AttributeSelector<'a> {
    Exists {
        attribute_name: WQName<'a>,
    },
    Matches {
        attribute_name: WQName<'a>,
        matcher: AttributeMatcher,
        value: Cow<'a, str>,
        modifier: AttributeModifier,
    },
}

impl<'a> CSSParse<'a> for AttributeSelector<'a> {
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
                value,
                modifier,
            }),
            None => Ok(AttributeSelector::Exists { attribute_name }),
        }
    }
}

fn parse_attribute_value_matcher<'a>(
    parser: &mut Parser<'a>,
) -> Result<(AttributeMatcher, Cow<'a, str>, AttributeModifier), ParseError> {
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

impl<'a> CSSValidateSelector for AttributeSelector<'a> {
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
