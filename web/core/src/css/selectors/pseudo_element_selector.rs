use super::{CSSValidateSelector, LegacyPseudoElementSelector, PseudoClassSelector};
use crate::css::{syntax::Token, CSSParse, ParseError, Parser};

/// <https://drafts.csswg.org/selectors-4/#typedef-pseudo-element-selector>
#[derive(Clone, Debug, PartialEq)]
pub enum PseudoElementSelector {
    PseudoClass(PseudoClassSelector),
    Legacy(LegacyPseudoElementSelector),
}

impl<'a> CSSParse<'a> for PseudoElementSelector {
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

impl CSSValidateSelector for PseudoElementSelector {
    fn is_valid(&self) -> bool {
        match self {
            Self::PseudoClass(pseudo_class_selector) => pseudo_class_selector.is_valid(),
            Self::Legacy(legacy_pseudo_element_selector) => {
                legacy_pseudo_element_selector.is_valid()
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PseudoElementSelector;
    use crate::css::{
        selectors::{AnyValue, LegacyPseudoElementSelector, PseudoClassSelector},
        syntax::Token,
        CSSParse,
    };

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
}
