//! <https://drafts.csswg.org/selectors-4/>

use std::borrow::Cow;

use crate::{
    parser::{CSSParse, ParseError, Parser},
    tokenizer::{HashFlag, Token},
};
/// <https://drafts.csswg.org/selectors-4/#typedef-ns-prefix>
#[derive(Clone, Debug)]
pub enum NSPrefix<'a> {
    Ident(Cow<'a, str>),
    Asterisk,
    Empty,
}

/// <https://drafts.csswg.org/selectors-4/#typedef-wq-name>
#[derive(Clone, Debug)]
pub struct WQName<'a> {
    pub prefix: NSPrefix<'a>,
    pub ident: Cow<'a, str>,
}

/// <https://drafts.csswg.org/selectors-4/#typedef-type-selector>
#[derive(Clone, Debug)]
pub enum TypeSelector<'a> {
    NSPrefix(NSPrefix<'a>),
    WQName(WQName<'a>),
}

/// <https://drafts.csswg.org/selectors-4/#typedef-id-selector>
#[derive(Clone, Debug)]
pub struct IDSelector<'a> {
    pub ident: Cow<'a, str>,
}

/// <https://drafts.csswg.org/selectors-4/#typedef-class-selector>
#[derive(Clone, Debug)]
pub struct ClassSelector<'a> {
    pub ident: Cow<'a, str>,
}

/// <https://drafts.csswg.org/selectors-4/#typedef-attr-matcher>
#[derive(Clone, Copy, Debug)]
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
#[derive(Clone, Copy, Debug, Default)]
pub enum AttributeModifier {
    /// `i`
    CaseInsensitive,

    /// `s`
    #[default]
    CaseSensitive,
}

/// <https://drafts.csswg.org/selectors-4/#typedef-attribute-selector>
#[derive(Clone, Debug)]
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

/// <https://w3c.github.io/csswg-drafts/css-syntax-3/#typedef-any-value>
#[derive(Clone, Debug)]
pub struct AnyValue<'a>(Vec<Token<'a>>);

/// <https://drafts.csswg.org/selectors-4/#typedef-pseudo-class-selector>
#[derive(Clone, Debug)]
pub enum PseudoClassSelector<'a> {
    Ident(Cow<'a, str>),
    Function {
        function_name: Cow<'a, str>,
        content: AnyValue<'a>,
    },
}

/// <https://drafts.csswg.org/selectors-4/#typedef-subclass-selector>
#[derive(Clone, Debug)]
pub enum SubClassSelector<'a> {
    ID(IDSelector<'a>),
    Class(ClassSelector<'a>),
    Attribute(AttributeSelector<'a>),
    PseudoClass(PseudoClassSelector<'a>),
}

/// <https://drafts.csswg.org/selectors-4/#typedef-compound-selector>
#[derive(Clone, Debug)]
pub struct CompoundSelector<'a>(Vec<CompoundSelectorPart<'a>>);

#[derive(Clone, Debug)]
pub struct CompoundSelectorPart<'a> {
    pub type_selector: Option<TypeSelector<'a>>,
    pub subclass_selectors: Vec<SubClassSelector<'a>>,
}

/// <https://drafts.csswg.org/selectors-4/#typedef-combinator>
#[derive(Clone, Copy, Debug, Default)]
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

/// <https://drafts.csswg.org/selectors-4/#typedef-complex-real-selector>
#[derive(Clone, Debug)]
pub struct ComplexRealSelector<'a> {
    pub first_selector: CompoundSelector<'a>,
    pub subsequent_selectors: Vec<(Combinator, CompoundSelector<'a>)>,
}
/// <https://drafts.csswg.org/selectors/#typedef-legacy-pseudo-element-selector>
#[derive(Clone, Copy, Debug)]
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

        if let Some(Token::Delim('|')) = parser.next_token() {
            Ok(prefix)
        } else {
            Err(ParseError)
        }
    }
}

impl<'a> CSSParse<'a> for WQName<'a> {
    // <https://drafts.csswg.org/selectors-4/#typedef-wq-name>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        let prefix = NSPrefix::parse(parser)?;

        if let Some(Token::Ident(ident)) = parser.next_token() {
            Ok(WQName { prefix, ident })
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
        if let Ok(ns_prefix) = NSPrefix::parse(parser) {
            if let Some(Token::Delim('*')) = parser.next_token() {
                return Ok(TypeSelector::NSPrefix(ns_prefix));
            }
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
            Some(Token::Delim('i')) => Ok(AttributeModifier::CaseInsensitive),
            Some(Token::Delim('s')) => Ok(AttributeModifier::CaseSensitive),
            _ => Err(ParseError),
        }
    }
}

impl<'a> CSSParse<'a> for AttributeSelector<'a> {
    // <https://drafts.csswg.org/selectors-4/#typedef-attribute-selector>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        if !matches!(parser.next_token(), Some(Token::BracketOpen)) {
            return Err(ParseError);
        }

        // Both variants start with a wqname
        let attribute_name = WQName::parse(parser)?;

        let value_matching_part = parser.parse_optional_value(parse_attribute_value_matcher);

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
    let attribute_value = match parser.next_token() {
        Some(Token::String(value) | Token::Ident(value)) => value,
        _ => return Err(ParseError),
    };
    let attribute_modifier = parser
        .parse_optional_value(AttributeModifier::parse)
        .unwrap_or_default();

    Ok((attribute_matcher, attribute_value, attribute_modifier))
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
                Ok(PseudoClassSelector::Function {
                    function_name,
                    content,
                })
            },
            _ => Err(ParseError),
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

impl<'a> CSSParse<'a> for Combinator {
    // <https://drafts.csswg.org/selectors-4/#typedef-combinator>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        match parser.next_token() {
            Some(Token::Delim('>')) => Ok(Combinator::Descendant),
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
    let combinator = parser
        .parse_optional_value(Combinator::parse)
        .unwrap_or_default();
    let selector = CompoundSelector::parse(parser)?;

    Ok((combinator, selector))
}
