use super::CSSValidateSelector;
use crate::css::{
    parser::{CSSParse, ParseError, Parser},
    tokenizer::Token,
};

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

impl CSSValidateSelector for AttributeModifier {}

#[cfg(test)]
mod tests {
    use super::AttributeModifier;
    use crate::css::parser::CSSParse;

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
}
