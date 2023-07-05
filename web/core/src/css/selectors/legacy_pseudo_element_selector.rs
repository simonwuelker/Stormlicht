use string_interner::{static_interned, static_str};

use super::{CSSValidateSelector, Selector, Specificity};
use crate::{
    css::{syntax::Token, CSSParse, ParseError, Parser},
    dom::{dom_objects::Element, DOMPtr},
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
            match ident {
                static_interned!("before") => Ok(LegacyPseudoElementSelector::Before),
                static_interned!("after") => Ok(LegacyPseudoElementSelector::After),
                static_interned!("first-line") => Ok(LegacyPseudoElementSelector::FirstLine),
                static_interned!("first-letter") => Ok(LegacyPseudoElementSelector::FirstLetter),
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

impl Selector for LegacyPseudoElementSelector {
    fn matches(&self, _element: &DOMPtr<Element>) -> bool {
        todo!()
    }

    fn specificity(&self) -> Specificity {
        Specificity::new(0, 0, 1)
    }
}

#[cfg(test)]
mod tests {
    use super::LegacyPseudoElementSelector;
    use crate::css::CSSParse;

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
