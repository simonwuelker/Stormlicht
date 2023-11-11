use std::fmt;

use crate::{
    css::{
        selectors::{CSSValidateSelector, Selector, Specificity},
        syntax::Token,
        CSSParse, ParseError, Parser, Serialize, Serializer,
    },
    dom::{dom_objects::Element, DOMPtr},
    static_interned,
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
        // Unimplemented
        false
    }

    fn specificity(&self) -> Specificity {
        Specificity::new(0, 0, 1)
    }
}

impl Serialize for LegacyPseudoElementSelector {
    fn serialize_to<T: Serializer>(&self, serializer: &mut T) -> fmt::Result {
        match self {
            Self::Before => serializer.serialize(":before"),
            Self::After => serializer.serialize(":after"),
            Self::FirstLine => serializer.serialize(":first-line"),
            Self::FirstLetter => serializer.serialize(":first-letter"),
        }
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
