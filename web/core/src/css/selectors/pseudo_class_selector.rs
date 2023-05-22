use std::borrow::Cow;

use super::{AnyValue, CSSValidateSelector};
use crate::css::{
    parser::{CSSParse, ParseError, Parser},
    tokenizer::Token,
};

/// <https://drafts.csswg.org/selectors-4/#typedef-pseudo-class-selector>
#[derive(Clone, Debug, PartialEq)]
pub enum PseudoClassSelector<'a> {
    Ident(Cow<'a, str>),
    Function {
        function_name: Cow<'a, str>,
        content: AnyValue<'a>,
    },
}

impl<'a> CSSParse<'a> for PseudoClassSelector<'a> {
    // <https://drafts.csswg.org/selectors-4/#typedef-pseudo-class-selector>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        if !matches!(parser.next_token(), Some(Token::Colon)) {
            return Err(ParseError);
        }

        match parser.next_token() {
            Some(Token::Ident(ident)) => {
                // LEGACY:
                // The <pseudo-class-selector> production excludes the <legacy-pseudo-element-selector> production. (That is, :before/etc must never be parsed as a pseudo-class)
                match ident.as_ref() {
                    "before" | "after" | "first-line" | "first-letter" => Err(ParseError),
                    _ => Ok(PseudoClassSelector::Ident(ident)),
                }
            },
            Some(Token::Function(function_name)) => {
                let content = AnyValue::parse(parser)?;
                if matches!(parser.next_token(), Some(Token::ParenthesisClose)) {
                    Ok(PseudoClassSelector::Function {
                        function_name,
                        content,
                    })
                } else {
                    Err(ParseError)
                }
            },
            _ => Err(ParseError),
        }
    }
}

impl<'a> CSSValidateSelector for PseudoClassSelector<'a> {
    fn is_valid(&self) -> bool {
        // We don't support *any* legacy pseudo class selectors
        // As per spec, we therefore treat them as invalid
        false
    }
}

#[cfg(test)]
mod tests {
    use super::PseudoClassSelector;
    use crate::css::{
        parser::{CSSParse, ParseError},
        selectors::AnyValue,
        tokenizer::Token,
    };

    #[test]
    fn parse_pseudo_class_selector() {
        assert_eq!(
            PseudoClassSelector::parse_from_str(":foo"),
            Ok(PseudoClassSelector::Ident("foo".into()))
        );
        assert_eq!(
            PseudoClassSelector::parse_from_str(":foo(bar)"),
            Ok(PseudoClassSelector::Function {
                function_name: "foo".into(),
                content: AnyValue(vec![Token::Ident("bar".into())])
            })
        );

        // For legacy compatibility reasons, the pseudo class production *excludes* the legacy pseudo element
        // production
        assert_eq!(
            PseudoClassSelector::parse_from_str(":before"),
            Err(ParseError)
        );
        assert_eq!(
            PseudoClassSelector::parse_from_str(":after"),
            Err(ParseError)
        );
        assert_eq!(
            PseudoClassSelector::parse_from_str(":first-line"),
            Err(ParseError)
        );
        assert_eq!(
            PseudoClassSelector::parse_from_str(":first-letter"),
            Err(ParseError)
        );
    }
}
