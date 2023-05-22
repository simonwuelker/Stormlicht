use super::CSSValidateSelector;
use crate::css::{
    parser::{CSSParse, ParseError, Parser},
    tokenizer::Token,
};

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

impl CSSValidateSelector for AttributeMatcher {}

#[cfg(test)]
mod tests {
    use super::AttributeMatcher;
    use crate::css::parser::CSSParse;

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
