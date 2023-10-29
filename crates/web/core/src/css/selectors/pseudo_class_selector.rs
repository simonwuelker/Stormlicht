use std::fmt;

use string_interner::{static_interned, static_str, InternedString};

use crate::{
    css::{
        selectors::{AnyValue, CSSValidateSelector, Selector, Specificity},
        syntax::Token,
        CSSParse, ParseError, Parser, Serialize, Serializer,
    },
    dom::{dom_objects::Element, DOMPtr},
};

/// <https://drafts.csswg.org/selectors-4/#pseudo-classes>
#[derive(Clone, Debug, PartialEq)]
pub enum PseudoClassSelector {
    Ident(InternedString),
    Function {
        function_name: InternedString,
        content: AnyValue,
    },
}

impl<'a> CSSParse<'a> for PseudoClassSelector {
    // <https://drafts.csswg.org/selectors-4/#typedef-pseudo-class-selector>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        if !matches!(parser.next_token(), Some(Token::Colon)) {
            return Err(ParseError);
        }

        match parser.next_token() {
            Some(Token::Ident(ident)) => {
                // LEGACY: The <pseudo-class-selector> production excludes the <legacy-pseudo-element-selector> production.
                //         (That is, :before/etc must never be parsed as a pseudo-class)
                match ident {
                    static_interned!("before")
                    | static_interned!("after")
                    | static_interned!("first-line")
                    | static_interned!("first-letter") => Err(ParseError),
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

impl Selector for PseudoClassSelector {
    fn matches(&self, _element: &DOMPtr<Element>) -> bool {
        log::warn!("FIXME: Pseudo Class selector matching");
        false
    }

    fn specificity(&self) -> Specificity {
        match self {
            Self::Ident(_) => Specificity::new(0, 1, 0),
            Self::Function { .. } => {
                // FIXME: Some pseudo classes have their own specificity rules
                Specificity::new(0, 1, 0)
            },
        }
    }
}

impl CSSValidateSelector for PseudoClassSelector {
    fn is_valid(&self) -> bool {
        // We don't support *any* legacy pseudo class selectors
        // As per spec, we therefore treat them as invalid
        false
    }
}

impl Serialize for PseudoClassSelector {
    fn serialize_to<T: Serializer>(&self, serializer: &mut T) -> fmt::Result {
        match self {
            Self::Ident(identifier) => serializer.serialize_identifier(&identifier.to_string()),
            Self::Function {
                function_name,
                content,
            } => {
                serializer.serialize_identifier(&function_name.to_string())?;

                serializer.serialize('(')?;

                // FIXME: Serialize content
                _ = content;

                serializer.serialize(')')?;

                Ok(())
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PseudoClassSelector;
    use crate::css::{selectors::AnyValue, syntax::Token, CSSParse, ParseError};

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
