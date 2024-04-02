use std::fmt;

use crate::{
    css::{
        selectors::{CSSValidateSelector, NamespacePrefix},
        syntax::Token,
        CSSParse, ParseError, Parser, Serialize, Serializer,
    },
    InternedString,
};

/// <https://drafts.csswg.org/selectors-4/#typedef-wq-name>
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WellQualifiedName {
    pub prefix: Option<NamespacePrefix>,
    pub ident: InternedString,
}

impl<'a> CSSParse<'a> for WellQualifiedName {
    // <https://drafts.csswg.org/selectors-4/#typedef-wq-name>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        let prefix = parser.parse_optional_value(NamespacePrefix::parse);

        parser.skip_whitespace();

        if let Some(Token::Ident(ident)) = parser.next_token() {
            Ok(WellQualifiedName { prefix, ident })
        } else {
            Err(ParseError)
        }
    }
}

impl CSSValidateSelector for WellQualifiedName {
    fn is_valid(&self) -> bool {
        true
    }
}

impl Serialize for WellQualifiedName {
    fn serialize_to<T: Serializer>(&self, serializer: &mut T) -> fmt::Result {
        // FIXME: serialize name space prefix
        serializer.serialize_identifier(&self.ident.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::WellQualifiedName;
    use crate::css::{selectors::NamespacePrefix, CSSParse};

    #[test]
    fn parse_wq_name() {
        assert_eq!(
            WellQualifiedName::parse_from_str("foo | bar"),
            Ok(WellQualifiedName {
                prefix: Some(NamespacePrefix::Ident("foo".into())),
                ident: "bar".into()
            })
        );

        assert_eq!(
            WellQualifiedName::parse_from_str("bar"),
            Ok(WellQualifiedName {
                prefix: None,
                ident: "bar".into()
            })
        );
    }
}
