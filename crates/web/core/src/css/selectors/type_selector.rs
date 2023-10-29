use std::fmt;

use crate::{
    css::{
        selectors::{CSSValidateSelector, NSPrefix, Selector, Specificity, WQName},
        syntax::Token,
        CSSParse, ParseError, Parser, Serialize, Serializer,
    },
    dom::{dom_objects::Element, DOMPtr},
};

/// <https://drafts.csswg.org/selectors-4/#type-selectors>
#[derive(Clone, Debug, PartialEq)]
pub enum TypeSelector {
    /// <https://drafts.csswg.org/selectors-4/#type-nmsp>
    NSPrefix(Option<NSPrefix>),
    WQName(WQName),
}

impl<'a> CSSParse<'a> for TypeSelector {
    // <https://drafts.csswg.org/selectors-4/#typedef-type-selector>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        let start_state = parser.state();
        if let Ok(wq_name) = WQName::parse(parser) {
            return Ok(TypeSelector::WQName(wq_name));
        }

        parser.set_state(start_state);
        let ns_prefix = parser.parse_optional_value(NSPrefix::parse);
        parser.skip_whitespace();
        if matches!(parser.next_token(), Some(Token::Delim('*'))) {
            Ok(TypeSelector::NSPrefix(ns_prefix))
        } else {
            Err(ParseError)
        }
    }
}

impl CSSValidateSelector for TypeSelector {
    fn is_valid(&self) -> bool {
        match self {
            Self::NSPrefix(ns_prefix) => !ns_prefix.as_ref().is_some_and(|n| n.is_valid()),
            Self::WQName(wq_name) => wq_name.is_valid(),
        }
    }
}

impl Selector for TypeSelector {
    fn matches(&self, element: &DOMPtr<Element>) -> bool {
        _ = element;
        match self {
            Self::NSPrefix(_) => false,
            Self::WQName(wq_name) => {
                wq_name.prefix.is_none() && wq_name.ident == element.borrow().local_name()
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
            Self::NSPrefix(ns_prefix) => {
                // FIXME: serialize ns prefix
                _ = ns_prefix;

                serializer.serialize('*')?;
                Ok(())
            },
            Self::WQName(wq_name) => serializer.serialize(*wq_name),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TypeSelector;
    use crate::css::{
        selectors::{NSPrefix, WQName},
        CSSParse,
    };

    #[test]
    fn parse_type_selector() {
        assert_eq!(
            TypeSelector::parse_from_str("foo | bar"),
            Ok(TypeSelector::WQName(WQName {
                prefix: Some(NSPrefix::Ident("foo".into())),
                ident: "bar".into()
            }))
        );

        assert_eq!(
            TypeSelector::parse_from_str("foo | *"),
            Ok(TypeSelector::NSPrefix(Some(NSPrefix::Ident("foo".into()))))
        );

        assert_eq!(
            TypeSelector::parse_from_str("*"),
            Ok(TypeSelector::NSPrefix(None))
        );
    }
}
