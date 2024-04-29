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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WellQualifiedName {
    pub prefix: Option<NamespacePrefix>,
    pub ident: InternedString,
}

impl WellQualifiedName {
    #[inline]
    #[must_use]
    pub const fn without_namespace(ident: InternedString) -> Self {
        Self {
            prefix: None,
            ident,
        }
    }
}

impl<'a> CSSParse<'a> for WellQualifiedName {
    // <https://drafts.csswg.org/selectors-4/#typedef-wq-name>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        let name = match parser.next_token_ignoring_whitespace() {
            Some(Token::Delim('|')) => {
                let Some(Token::Ident(ident)) = parser.next_token_ignoring_whitespace() else {
                    return Err(ParseError);
                };

                Self {
                    prefix: None,
                    ident,
                }
            },
            Some(Token::Delim('*')) => {
                parser.expect_token(Token::Delim('|'))?;

                let Some(Token::Ident(ident)) = parser.next_token_ignoring_whitespace() else {
                    return Err(ParseError);
                };

                Self {
                    prefix: Some(NamespacePrefix::Asterisk),
                    ident,
                }
            },
            Some(Token::Ident(ident)) => {
                if matches!(
                    parser.peek_token_ignoring_whitespace(0),
                    Some(Token::Delim('|'))
                ) {
                    _ = parser.next_token_ignoring_whitespace();
                    // The identifier was the namespace prefix
                    let Some(Token::Ident(local_name)) = parser.next_token_ignoring_whitespace()
                    else {
                        return Err(ParseError);
                    };

                    Self {
                        prefix: Some(NamespacePrefix::Ident(ident)),
                        ident: local_name,
                    }
                } else {
                    // No namespace prefix
                    Self {
                        prefix: None,
                        ident,
                    }
                }
            },
            _ => return Err(ParseError),
        };

        Ok(name)
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
