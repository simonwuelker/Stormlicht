use super::{CSSValidateSelector, LegacyPseudoElementSelector, PseudoClassSelector};
use crate::css::{
    parser::{CSSParse, ParseError, Parser},
    tokenizer::Token,
};

/// <https://drafts.csswg.org/selectors-4/#typedef-pseudo-element-selector>
#[derive(Clone, Debug, PartialEq)]
pub enum PseudoElementSelector<'a> {
    PseudoClass(PseudoClassSelector<'a>),
    Legacy(LegacyPseudoElementSelector),
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

impl<'a> CSSValidateSelector for PseudoElementSelector<'a> {
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
        parser::CSSParse,
        selectors::{AnyValue, LegacyPseudoElementSelector, PseudoClassSelector},
        tokenizer::Token,
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
