use std::fmt;

use super::CSSValidateSelector;
use crate::css::{syntax::Token, CSSParse, ParseError, Parser, Serialize, Serializer};

/// <https://drafts.csswg.org/selectors-4/#typedef-attr-matcher>
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

impl<'a> CSSParse<'a> for AttributeMatcher {
    // <https://drafts.csswg.org/selectors-4/#typedef-attr-matcher>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        let attribute_matcher = match parser.next_token_ignoring_whitespace() {
            Some(Token::Delim('~')) => AttributeMatcher::WhiteSpaceSeperatedListContaining,
            Some(Token::Delim('|')) => AttributeMatcher::HyphenSeperatedListBeginningWith,
            Some(Token::Delim('^')) => AttributeMatcher::StartsWith,
            Some(Token::Delim('$')) => AttributeMatcher::EndsWith,
            Some(Token::Delim('*')) => AttributeMatcher::ContainsSubstring,
            Some(Token::Delim('=')) => return Ok(Self::EqualTo),
            _ => return Err(ParseError),
        };

        // No whitespace allowed here
        if let Some(Token::Delim('=')) = parser.next_token() {
            Ok(attribute_matcher)
        } else {
            Err(ParseError)
        }
    }
}

impl CSSValidateSelector for AttributeMatcher {
    fn is_valid(&self) -> bool {
        // We don't support *any* attribute matchers
        // As per spec, we therefore treat them as invalid
        false
    }
}

impl Serialize for AttributeMatcher {
    fn serialize_to<T: Serializer>(&self, serializer: &mut T) -> fmt::Result {
        match self {
            Self::EqualTo => serializer.serialize('='),
            Self::HyphenSeperatedListBeginningWith => serializer.serialize("|="),
            Self::WhiteSpaceSeperatedListContaining => serializer.serialize("~="),
            Self::StartsWith => serializer.serialize("^="),
            Self::EndsWith => serializer.serialize("$="),
            Self::ContainsSubstring => serializer.serialize("*="),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AttributeMatcher;
    use crate::css::CSSParse;

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
}
