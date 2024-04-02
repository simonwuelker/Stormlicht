use std::fmt;

use crate::{
    css::{
        selectors::{
            CSSValidateSelector, NamespacePrefix, Selector, Specificity, WellQualifiedName,
        },
        syntax::Token,
        CSSParse, ParseError, Parser, Serialize, Serializer,
    },
    dom::{dom_objects::Element, DomPtr},
};

/// <https://drafts.csswg.org/selectors-4/#type-selectors>
#[derive(Clone, Debug, PartialEq)]
pub enum TypeSelector {
    /// <https://drafts.csswg.org/selectors-4/#the-universal-selector>
    Universal(Option<NamespacePrefix>),
    Typename(WellQualifiedName),
}

impl<'a> CSSParse<'a> for TypeSelector {
    // <https://drafts.csswg.org/selectors-4/#typedef-type-selector>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        let start_state = parser.state();
        if let Ok(wq_name) = WellQualifiedName::parse(parser) {
            return Ok(TypeSelector::Typename(wq_name));
        }

        parser.set_state(start_state);
        let ns_prefix = parser
            .parse_optional_value(Option::<NamespacePrefix>::parse)
            .flatten();
        parser.skip_whitespace();
        if matches!(parser.next_token(), Some(Token::Delim('*'))) {
            Ok(TypeSelector::Universal(ns_prefix))
        } else {
            Err(ParseError)
        }
    }
}

impl CSSValidateSelector for TypeSelector {
    fn is_valid(&self) -> bool {
        match self {
            Self::Universal(namespace) => !namespace.as_ref().is_some_and(|n| n.is_valid()),
            Self::Typename(type_name) => type_name.is_valid(),
        }
    }
}

impl Selector for TypeSelector {
    fn matches(&self, element: &DomPtr<Element>) -> bool {
        match self {
            Self::Universal(namespace) => {
                // This is the universal selector
                // FIXME: If there is a namespace then we should only match elements from that
                //        namespace
                _ = namespace;
                true
            },
            Self::Typename(type_name) => {
                type_name.prefix.is_none() && type_name.ident == element.borrow().local_name()
            },
        }
    }

    fn specificity(&self) -> Specificity {
        Specificity::new(0, 0, 1)
    }
}

impl Serialize for TypeSelector {
    fn serialize_to<T: Serializer>(&self, serializer: &mut T) -> fmt::Result {
        match self {
            Self::Universal(namespace) => {
                // FIXME: serialize ns prefix
                _ = namespace;

                serializer.serialize('*')?;
                Ok(())
            },
            Self::Typename(type_name) => serializer.serialize(*type_name),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TypeSelector;
    use crate::css::{
        selectors::{NamespacePrefix, WellQualifiedName},
        CSSParse,
    };

    #[test]
    fn parse_type_selector() {
        assert_eq!(
            TypeSelector::parse_from_str("foo | bar"),
            Ok(TypeSelector::Typename(WellQualifiedName {
                prefix: Some(NamespacePrefix::Ident("foo".into())),
                ident: "bar".into()
            }))
        );

        assert_eq!(
            TypeSelector::parse_from_str("foo | *"),
            Ok(TypeSelector::Universal(Some(NamespacePrefix::Ident(
                "foo".into()
            ))))
        );

        assert_eq!(
            TypeSelector::parse_from_str("*"),
            Ok(TypeSelector::Universal(None))
        );
    }
}
