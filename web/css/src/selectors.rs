//! <https://drafts.csswg.org/selectors-4/>

use std::borrow::Cow;

use crate::{
    parser::{CSSParse, ParseError, Parser},
    tokenizer::{HashFlag, Token},
};

/// <https://drafts.csswg.org/selectors-4/#typedef-complex-selector>
#[derive(Clone, Debug, PartialEq)]
pub struct ComplexSelector<'a> {
    pub first_unit: ComplexSelectorUnit<'a>,
    pub subsequent_units: Vec<(Combinator, ComplexSelectorUnit<'a>)>,
}

/// <https://drafts.csswg.org/selectors-4/#typedef-complex-selector-unit>
#[derive(Clone, Debug, PartialEq)]
pub struct ComplexSelectorUnit<'a>(pub Vec<ComplexSelectorUnitPart<'a>>);

#[derive(Clone, Debug, PartialEq)]
pub struct ComplexSelectorUnitPart<'a> {
    pub compound_selector: Option<CompoundSelector<'a>>,
    pub pseudo_compound_selectors: Vec<PseudoCompoundSelector<'a>>,
}

/// <https://drafts.csswg.org/selectors-4/#typedef-complex-real-selector>
#[derive(Clone, Debug, PartialEq)]
pub struct ComplexRealSelector<'a> {
    pub first_selector: CompoundSelector<'a>,
    pub subsequent_selectors: Vec<(Combinator, CompoundSelector<'a>)>,
}

/// <https://drafts.csswg.org/selectors-4/#typedef-compound-selector>
#[derive(Clone, Debug, PartialEq)]
pub struct CompoundSelector<'a>(Vec<CompoundSelectorPart<'a>>);

#[derive(Clone, Debug, PartialEq)]
pub struct CompoundSelectorPart<'a> {
    pub type_selector: Option<TypeSelector<'a>>,
    pub subclass_selectors: Vec<SubClassSelector<'a>>,
}

/// <https://drafts.csswg.org/selectors-4/#typedef-pseudo-compound-selector>
#[derive(Clone, Debug, PartialEq)]
pub struct PseudoCompoundSelector<'a> {
    pub pseudo_element_selector: PseudoElementSelector<'a>,
    pub pseudo_class_selectors: Vec<PseudoClassSelector<'a>>,
}

/// <https://drafts.csswg.org/selectors-4/#typedef-simple-selector>
#[derive(Clone, Debug, PartialEq)]
pub enum SimpleSelector<'a> {
    Type(TypeSelector<'a>),
    SubClass(SubClassSelector<'a>),
}

/// <https://drafts.csswg.org/selectors-4/#typedef-combinator>
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Combinator {
    /// ` `
    ///
    /// # Specification
    /// <https://drafts.csswg.org/selectors-4/#descendant-combinators>
    #[default]
    Descendant,

    /// `>`
    ///
    /// # Specification
    /// <https://drafts.csswg.org/selectors-4/#child-combinators>
    Child,

    /// `+`
    ///
    /// # Specification
    /// <https://drafts.csswg.org/selectors-4/#adjacent-sibling-combinators>
    NextSibling,

    /// `~`
    ///
    /// # Specification
    /// <https://drafts.csswg.org/selectors-4/#general-sibling-combinators>
    SubsequentSibling,

    /// `||`
    ///
    /// # Specification
    /// <https://drafts.csswg.org/selectors-4/#the-column-combinator>
    Column,
}

/// <https://drafts.csswg.org/selectors-4/#typedef-wq-name>
#[derive(Clone, Debug, PartialEq)]
pub struct WQName<'a> {
    pub prefix: Option<NSPrefix<'a>>,
    pub ident: Cow<'a, str>,
}

/// <https://drafts.csswg.org/selectors-4/#typedef-ns-prefix>
#[derive(Clone, Debug, PartialEq)]
pub enum NSPrefix<'a> {
    Ident(Cow<'a, str>),
    Asterisk,
    Empty,
}

/// <https://drafts.csswg.org/selectors-4/#typedef-type-selector>
#[derive(Clone, Debug, PartialEq)]
pub enum TypeSelector<'a> {
    NSPrefix(Option<NSPrefix<'a>>),
    WQName(WQName<'a>),
}

/// <https://drafts.csswg.org/selectors-4/#typedef-subclass-selector>
#[derive(Clone, Debug, PartialEq)]
pub enum SubClassSelector<'a> {
    ID(IDSelector<'a>),
    Class(ClassSelector<'a>),
    Attribute(AttributeSelector<'a>),
    PseudoClass(PseudoClassSelector<'a>),
}
/// <https://drafts.csswg.org/selectors-4/#typedef-id-selector>
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IDSelector<'a> {
    pub ident: Cow<'a, str>,
}

/// <https://drafts.csswg.org/selectors-4/#typedef-class-selector>
#[derive(Clone, Debug, PartialEq)]
pub struct ClassSelector<'a> {
    pub ident: Cow<'a, str>,
}

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

/// <https://drafts.csswg.org/selectors-4/#typedef-attr-matcher>
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AttributeMatcher {
    /// `~=`
    WhiteSpaceSeperatedListContaining,

    /// `|=`
    HyphenSeperatedListBeginningWith,

    /// `^`
    StartsWith,

    /// `$=`
    EndsWith,

    /// `*=`
    ContainsSubstring,

    /// `=`
    EqualTo,
}

/// <https://drafts.csswg.org/selectors-4/#typedef-attr-modifier>
///
/// See also: [Case Sensitivity](https://drafts.csswg.org/selectors-4/#attribute-case)
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum AttributeModifier {
    /// `i`
    CaseInsensitive,

    /// `s`
    #[default]
    CaseSensitive,
}

/// <https://drafts.csswg.org/selectors-4/#typedef-pseudo-class-selector>
#[derive(Clone, Debug, PartialEq)]
pub enum PseudoClassSelector<'a> {
    Ident(Cow<'a, str>),
    Function {
        function_name: Cow<'a, str>,
        content: AnyValue<'a>,
    },
}

/// <https://drafts.csswg.org/selectors-4/#typedef-pseudo-element-selector>
#[derive(Clone, Debug, PartialEq)]
pub enum PseudoElementSelector<'a> {
    PseudoClass(PseudoClassSelector<'a>),
    Legacy(LegacyPseudoElementSelector),
}

/// <https://drafts.csswg.org/selectors/#typedef-legacy-pseudo-element-selector>
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LegacyPseudoElementSelector {
    /// `:before`
    Before,
    /// `:after`
    After,
    /// `:first-line`
    FirstLine,
    /// `:first-letter`
    FirstLetter,
}
/// <https://w3c.github.io/csswg-drafts/css-syntax-3/#typedef-any-value>
#[derive(Clone, Debug, PartialEq)]
pub struct AnyValue<'a>(Vec<Token<'a>>);

impl<'a> CSSParse<'a> for ComplexSelector<'a> {
    // <https://drafts.csswg.org/selectors-4/#typedef-complex-selector>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        let first_unit = ComplexSelectorUnit::parse(parser)?;

        let subsequent_units =
            parser.parse_any_number_of(parse_complex_selector_unit_with_combinator);

        Ok(ComplexSelector {
            first_unit,
            subsequent_units,
        })
    }
}

fn parse_complex_selector_unit_with_combinator<'a>(
    parser: &mut Parser<'a>,
) -> Result<(Combinator, ComplexSelectorUnit<'a>), ParseError> {
    parser.skip_whitespace();
    let combinator = parser
        .parse_optional_value(Combinator::parse)
        .unwrap_or_default();
    parser.skip_whitespace();
    let complex_selector_unit = ComplexSelectorUnit::parse(parser)?;

    Ok((combinator, complex_selector_unit))
}

impl<'a> CSSParse<'a> for ComplexSelectorUnit<'a> {
    // <https://drafts.csswg.org/selectors-4/#typedef-complex-selector-unit>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        let complex_selector_unit_parts =
            parser.parse_nonempty_comma_seperated_list(parse_complex_selector_unit_part)?;
        Ok(ComplexSelectorUnit(complex_selector_unit_parts))
    }
}

fn parse_complex_selector_unit_part<'a>(
    parser: &mut Parser<'a>,
) -> Result<ComplexSelectorUnitPart<'a>, ParseError> {
    let compound_selector = parser.parse_optional_value(CompoundSelector::parse);
    let pseudo_compound_selectors = parser.parse_any_number_of(PseudoCompoundSelector::parse);
    Ok(ComplexSelectorUnitPart {
        compound_selector,
        pseudo_compound_selectors,
    })
}

impl<'a> CSSParse<'a> for ComplexRealSelector<'a> {
    // <https://drafts.csswg.org/selectors-4/#typedef-complex-real-selector>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        let first_selector = CompoundSelector::parse(parser)?;

        let subsequent_selectors = parser.parse_any_number_of(parse_selector_with_combinator);

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

impl<'a> CSSParse<'a> for CompoundSelector<'a> {
    // <https://drafts.csswg.org/selectors-4/#typedef-compound-selector>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        let compound_selector_parts = parser.parse_nonempty_comma_seperated_list(|parser| {
            let type_selector = parser.parse_optional_value(TypeSelector::parse);
            let subclass_selectors = parser.parse_any_number_of(SubClassSelector::parse);

            Ok(CompoundSelectorPart {
                type_selector,
                subclass_selectors,
            })
        })?;
        Ok(CompoundSelector(compound_selector_parts))
    }
}

impl<'a> CSSParse<'a> for PseudoCompoundSelector<'a> {
    // <https://drafts.csswg.org/selectors-4/#typedef-pseudo-compound-selector>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        let pseudo_element_selector = PseudoElementSelector::parse(parser)?;

        let pseudo_class_selectors = parser.parse_any_number_of(PseudoClassSelector::parse);
        Ok(PseudoCompoundSelector {
            pseudo_element_selector,
            pseudo_class_selectors,
        })
    }
}

impl<'a> CSSParse<'a> for SimpleSelector<'a> {
    // <https://drafts.csswg.org/selectors-4/#typedef-simple-selector>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        let start_state = parser.state();
        if let Ok(type_selector) = TypeSelector::parse(parser) {
            return Ok(SimpleSelector::Type(type_selector));
        }

        parser.set_state(start_state);
        if let Ok(subclass_selector) = SubClassSelector::parse(parser) {
            return Ok(SimpleSelector::SubClass(subclass_selector));
        }

        Err(ParseError)
    }
}

impl<'a> CSSParse<'a> for Combinator {
    // <https://drafts.csswg.org/selectors-4/#typedef-combinator>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        match parser.next_token() {
            Some(Token::Delim('>')) => Ok(Combinator::Child),
            Some(Token::Delim('+')) => Ok(Combinator::NextSibling),
            Some(Token::Delim('~')) => Ok(Combinator::SubsequentSibling),
            Some(Token::Delim('|')) => {
                if matches!(parser.next_token(), Some(Token::Delim('|'))) {
                    Ok(Combinator::Column)
                } else {
                    Err(ParseError)
                }
            },
            _ => Err(ParseError),
        }
    }
}

impl<'a> CSSParse<'a> for WQName<'a> {
    // <https://drafts.csswg.org/selectors-4/#typedef-wq-name>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        let prefix = parser.parse_optional_value(NSPrefix::parse);

        parser.skip_whitespace();

        if let Some(Token::Ident(ident)) = parser.next_token() {
            Ok(WQName { prefix, ident })
        } else {
            Err(ParseError)
        }
    }
}

impl<'a> CSSParse<'a> for NSPrefix<'a> {
    // <https://drafts.csswg.org/selectors-4/#typedef-ns-prefix>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        let prefix = parser
            .parse_optional_value(|parser| match parser.next_token() {
                Some(Token::Ident(ident)) => Ok(NSPrefix::Ident(ident)),
                Some(Token::Delim('*')) => Ok(NSPrefix::Asterisk),
                _ => Err(ParseError),
            })
            .unwrap_or(NSPrefix::Empty);

        parser.skip_whitespace();

        if let Some(Token::Delim('|')) = parser.next_token() {
            Ok(prefix)
        } else {
            Err(ParseError)
        }
    }
}

impl<'a> CSSParse<'a> for TypeSelector<'a> {
    // <https://drafts.csswg.org/selectors-4/#typedef-type-selector>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        let start_state = parser.state();
        if let Ok(wq_name) = WQName::parse(parser) {
            return Ok(TypeSelector::WQName(wq_name));
        }

        parser.set_state(start_state);
        let ns_prefix = parser.parse_optional_value(NSPrefix::parse);
        parser.skip_whitespace();
        if matches!(parser.next_token(), Some(Token::Delim('*'))) {
            return Ok(TypeSelector::NSPrefix(ns_prefix));
        } else {
            Err(ParseError)
        }
    }
}

impl<'a> CSSParse<'a> for SubClassSelector<'a> {
    // <https://drafts.csswg.org/selectors-4/#typedef-subclass-selector>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        let start_state = parser.state();
        if let Ok(id_selector) = IDSelector::parse(parser) {
            return Ok(SubClassSelector::ID(id_selector));
        }

        parser.set_state(start_state);
        if let Ok(class_selector) = ClassSelector::parse(parser) {
            return Ok(SubClassSelector::Class(class_selector));
        }

        parser.set_state(start_state);
        if let Ok(attribute_selector) = AttributeSelector::parse(parser) {
            return Ok(SubClassSelector::Attribute(attribute_selector));
        }

        parser.set_state(start_state);
        if let Ok(pseudoclass_selector) = PseudoClassSelector::parse(parser) {
            return Ok(SubClassSelector::PseudoClass(pseudoclass_selector));
        }

        Err(ParseError)
    }
}

impl<'a> CSSParse<'a> for IDSelector<'a> {
    // <https://drafts.csswg.org/selectors-4/#typedef-id-selector>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        if let Some(Token::Hash(ident, HashFlag::Id)) = parser.next_token() {
            Ok(IDSelector { ident })
        } else {
            Err(ParseError)
        }
    }
}

impl<'a> CSSParse<'a> for ClassSelector<'a> {
    // <https://drafts.csswg.org/selectors-4/#typedef-class-selector>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        if let Some(Token::Delim('.')) = parser.next_token() {
            if let Some(Token::Ident(ident)) = parser.next_token() {
                return Ok(ClassSelector { ident });
            }
        }
        Err(ParseError)
    }
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

impl<'a> CSSParse<'a> for AttributeMatcher {
    // <https://drafts.csswg.org/selectors-4/#typedef-attr-matcher>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        let attribute_matcher = parser
            .parse_optional_value(|parser| match parser.next_token() {
                Some(Token::Delim('~')) => Ok(AttributeMatcher::WhiteSpaceSeperatedListContaining),
                Some(Token::Delim('|')) => Ok(AttributeMatcher::HyphenSeperatedListBeginningWith),
                Some(Token::Delim('^')) => Ok(AttributeMatcher::StartsWith),
                Some(Token::Delim('$')) => Ok(AttributeMatcher::EndsWith),
                Some(Token::Delim('*')) => Ok(AttributeMatcher::ContainsSubstring),
                _ => Err(ParseError),
            })
            .unwrap_or(AttributeMatcher::EqualTo);

        if let Some(Token::Delim('=')) = parser.next_token() {
            Ok(attribute_matcher)
        } else {
            Err(ParseError)
        }
    }
}

impl<'a> CSSParse<'a> for AttributeModifier {
    // <https://drafts.csswg.org/selectors-4/#typedef-attr-modifier>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        match parser.next_token() {
            Some(Token::Ident(ident)) => match ident.as_ref() {
                "i" => Ok(AttributeModifier::CaseInsensitive),
                "s" => Ok(AttributeModifier::CaseSensitive),
                _ => Err(ParseError),
            },
            _ => Err(ParseError),
        }
    }
}

impl<'a> CSSParse<'a> for PseudoClassSelector<'a> {
    // <https://drafts.csswg.org/selectors-4/#typedef-pseudo-class-selector>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        if !matches!(parser.next_token(), Some(Token::Colon)) {
            return Err(ParseError);
        }

        match parser.next_token() {
            Some(Token::Ident(ident)) => Ok(PseudoClassSelector::Ident(ident)),
            Some(Token::Function(function_name)) => {
                let content = AnyValue::parse(parser)?;
                if matches!(parser.next_token(), Some(Token::ParenthesisClose)) {
                    Ok(PseudoClassSelector::Function {
                        function_name,
                        content,
                    })
                } else {
                    Err(ParseError)
                }
            },
            _ => Err(ParseError),
        }
    }
}

impl<'a> CSSParse<'a> for PseudoElementSelector<'a> {
    // <https://drafts.csswg.org/selectors-4/#typedef-pseudo-element-selector>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        let start_state = parser.state();
        if matches!(parser.next_token(), Some(Token::Colon)) {
            if let Ok(pseudo_class_selector) = PseudoClassSelector::parse(parser) {
                return Ok(PseudoElementSelector::PseudoClass(pseudo_class_selector));
            }
        }

        parser.set_state(start_state);
        if let Ok(legacy_pseudo_element_selector) = LegacyPseudoElementSelector::parse(parser) {
            return Ok(PseudoElementSelector::Legacy(
                legacy_pseudo_element_selector,
            ));
        }

        Err(ParseError)
    }
}

impl<'a> CSSParse<'a> for LegacyPseudoElementSelector {
    // <https://drafts.csswg.org/selectors-4/#typedef-legacy-pseudo-element-selector>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        if !matches!(parser.next_token(), Some(Token::Colon)) {
            return Err(ParseError);
        }

        if let Some(Token::Ident(ident)) = parser.next_token() {
            match ident.as_ref() {
                "before" => Ok(LegacyPseudoElementSelector::Before),
                "after" => Ok(LegacyPseudoElementSelector::After),
                "first-line" => Ok(LegacyPseudoElementSelector::FirstLine),
                "first-letter" => Ok(LegacyPseudoElementSelector::FirstLetter),
                _ => Err(ParseError),
            }
        } else {
            Err(ParseError)
        }
    }
}

impl<'a> CSSParse<'a> for AnyValue<'a> {
    // <https://w3c.github.io/csswg-drafts/css-syntax-3/#typedef-any-value>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        let mut values = vec![];

        let mut parenthesis_balance = 0;
        let mut bracket_balance = 0;
        let mut curly_brace_balance = 0;
        let mut state_before_ending_token;

        loop {
            state_before_ending_token = parser.state();

            match parser.next_token() {
                None | Some(Token::BadString(_)) | Some(Token::BadURI(_)) => break,
                Some(Token::ParenthesisOpen) => parenthesis_balance += 1,
                Some(Token::BracketOpen) => bracket_balance += 1,
                Some(Token::CurlyBraceOpen) => curly_brace_balance += 1,
                Some(Token::ParenthesisClose) => {
                    if parenthesis_balance == 0 {
                        break;
                    } else {
                        parenthesis_balance -= 1;
                    }
                },
                Some(Token::BracketClose) => {
                    if bracket_balance == 0 {
                        break;
                    } else {
                        bracket_balance -= 1;
                    }
                },
                Some(Token::CurlyBraceClose) => {
                    if curly_brace_balance == 0 {
                        break;
                    } else {
                        curly_brace_balance -= 1;
                    }
                },
                Some(other) => values.push(other),
            }
        }
        parser.set_state(state_before_ending_token);

        if values.is_empty() {
            return Err(ParseError);
        }

        Ok(Self(values))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_selector() {
        assert_eq!(
            SimpleSelector::parse_from_str("foo"),
            Ok(SimpleSelector::Type(TypeSelector::WQName(WQName {
                prefix: None,
                ident: "foo".into()
            })))
        );

        assert_eq!(
            SimpleSelector::parse_from_str("#foo"),
            Ok(SimpleSelector::SubClass(SubClassSelector::ID(IDSelector {
                ident: "foo".into()
            })))
        );
    }

    #[test]
    fn parse_combinator() {
        assert_eq!(Combinator::parse_from_str(">"), Ok(Combinator::Child));
        assert_eq!(Combinator::parse_from_str("+"), Ok(Combinator::NextSibling));
        assert_eq!(
            Combinator::parse_from_str("~"),
            Ok(Combinator::SubsequentSibling)
        );
        assert_eq!(Combinator::parse_from_str("||"), Ok(Combinator::Column));
    }

    #[test]
    fn parse_wq_name() {
        assert_eq!(
            WQName::parse_from_str("foo | bar"),
            Ok(WQName {
                prefix: Some(NSPrefix::Ident("foo".into())),
                ident: "bar".into()
            })
        );

        assert_eq!(
            WQName::parse_from_str("bar"),
            Ok(WQName {
                prefix: None,
                ident: "bar".into()
            })
        );
    }

    #[test]
    fn parse_ns_prefix() {
        assert_eq!(
            NSPrefix::parse_from_str("foo |"),
            Ok(NSPrefix::Ident("foo".into()))
        );
        assert_eq!(NSPrefix::parse_from_str("* |"), Ok(NSPrefix::Asterisk));
        assert_eq!(NSPrefix::parse_from_str("|"), Ok(NSPrefix::Empty),);
    }

    #[test]
    fn parse_type_selector() {
        assert_eq!(
            TypeSelector::parse_from_str("foo | bar"),
            Ok(TypeSelector::WQName(WQName {
                prefix: Some(NSPrefix::Ident("foo".into())),
                ident: "bar".into()
            }))
        );

        assert_eq!(
            TypeSelector::parse_from_str("foo | *"),
            Ok(TypeSelector::NSPrefix(Some(NSPrefix::Ident("foo".into()))))
        );

        assert_eq!(
            TypeSelector::parse_from_str("*"),
            Ok(TypeSelector::NSPrefix(None))
        );
    }

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

    #[test]
    fn parse_attribute_matcher() {
        assert_eq!(
            AttributeMatcher::parse_from_str("="),
            Ok(AttributeMatcher::EqualTo)
        );
        assert_eq!(
            AttributeMatcher::parse_from_str("|="),
            Ok(AttributeMatcher::HyphenSeperatedListBeginningWith)
        );
        assert_eq!(
            AttributeMatcher::parse_from_str("~="),
            Ok(AttributeMatcher::WhiteSpaceSeperatedListContaining)
        );
        assert_eq!(
            AttributeMatcher::parse_from_str("^="),
            Ok(AttributeMatcher::StartsWith)
        );
        assert_eq!(
            AttributeMatcher::parse_from_str("$="),
            Ok(AttributeMatcher::EndsWith)
        );
        assert_eq!(
            AttributeMatcher::parse_from_str("*="),
            Ok(AttributeMatcher::ContainsSubstring)
        );
    }

    #[test]
    fn parse_attribute_modifier() {
        assert_eq!(
            AttributeModifier::parse_from_str("i"),
            Ok(AttributeModifier::CaseInsensitive)
        );
        assert_eq!(
            AttributeModifier::parse_from_str("s"),
            Ok(AttributeModifier::CaseSensitive)
        );
    }

    #[test]
    fn parse_pseudo_class_selector() {
        assert_eq!(
            PseudoClassSelector::parse_from_str(":foo"),
            Ok(PseudoClassSelector::Ident("foo".into()))
        );
        assert_eq!(
            PseudoClassSelector::parse_from_str(":foo(bar)"),
            Ok(PseudoClassSelector::Function {
                function_name: "foo".into(),
                content: AnyValue(vec![Token::Ident("bar".into())])
            })
        );
    }

    #[test]
    fn parse_pseudo_element_selector() {
        assert_eq!(
            PseudoElementSelector::parse_from_str("::foo"),
            Ok(PseudoElementSelector::PseudoClass(
                PseudoClassSelector::Ident("foo".into())
            ))
        );
        assert_eq!(
            PseudoElementSelector::parse_from_str("::foo(bar)"),
            Ok(PseudoElementSelector::PseudoClass(
                PseudoClassSelector::Function {
                    function_name: "foo".into(),
                    content: AnyValue(vec![Token::Ident("bar".into())])
                }
            ))
        );
        assert_eq!(
            PseudoElementSelector::parse_from_str(":before"),
            Ok(PseudoElementSelector::Legacy(
                LegacyPseudoElementSelector::Before
            ))
        );
    }

    #[test]
    fn parse_legacy_pseudo_element_selector() {
        assert_eq!(
            LegacyPseudoElementSelector::parse_from_str(":before"),
            Ok(LegacyPseudoElementSelector::Before)
        );
        assert_eq!(
            LegacyPseudoElementSelector::parse_from_str(":after"),
            Ok(LegacyPseudoElementSelector::After)
        );
        assert_eq!(
            LegacyPseudoElementSelector::parse_from_str(":first-line"),
            Ok(LegacyPseudoElementSelector::FirstLine)
        );
        assert_eq!(
            LegacyPseudoElementSelector::parse_from_str(":first-letter"),
            Ok(LegacyPseudoElementSelector::FirstLetter)
        );
    }
}
