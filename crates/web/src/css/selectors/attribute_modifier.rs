use super::CSSValidateSelector;
use crate::{
    css::{syntax::Token, CSSParse, ParseError, Parser},
    static_interned,
};

/// <https://drafts.csswg.org/selectors-4/#typedef-attr-modifier>
///
/// See also: [Case Sensitivity](https://drafts.csswg.org/selectors-4/#attribute-case)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AttributeModifier {
    /// `i`
    CaseInsensitive,

    /// `s`
    #[default]
    CaseSensitive,
}

impl AttributeModifier {
    pub fn is_case_insensitive(&self) -> bool {
        *self == Self::CaseInsensitive
    }
}

impl<'a> CSSParse<'a> for AttributeModifier {
    // <https://drafts.csswg.org/selectors-4/#typedef-attr-modifier>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        match parser.next_token_ignoring_whitespace() {
            Some(Token::Ident(ident)) => match ident {
                static_interned!("i") => Ok(AttributeModifier::CaseInsensitive),
                static_interned!("s") => Ok(AttributeModifier::CaseSensitive),
                _ => Err(ParseError),
            },
            _ => Err(ParseError),
        }
    }
}

impl CSSValidateSelector for AttributeModifier {
    fn is_valid(&self) -> bool {
        // We don't support *any* attribute modifiers
        // As per spec, we therefore treat them as invalid
        false
    }
}

#[cfg(test)]
mod tests {
    use super::AttributeModifier;
    use crate::css::CSSParse;

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
