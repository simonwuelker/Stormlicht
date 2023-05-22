use super::CSSValidateSelector;
use crate::css::{
    parser::{CSSParse, ParseError, Parser},
    tokenizer::Token,
};

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

impl CSSValidateSelector for LegacyPseudoElementSelector {
    fn is_valid(&self) -> bool {
        // We don't support *any* legacy pseudo element selectors
        // As per spec, we therefore treat them as invalid
        false
    }
}

#[cfg(test)]
mod tests {
    use super::LegacyPseudoElementSelector;
    use crate::css::parser::CSSParse;

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
