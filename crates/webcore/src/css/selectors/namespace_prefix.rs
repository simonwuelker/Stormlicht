use crate::{
    css::{selectors::CSSValidateSelector, syntax::Token, CSSParse, ParseError, Parser},
    InternedString,
};

/// <https://drafts.csswg.org/selectors-4/#typedef-ns-prefix>
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NamespacePrefix {
    Ident(InternedString),
    Asterisk,
}

impl<'a> CSSParse<'a> for Option<NamespacePrefix> {
    /// <https://drafts.csswg.org/selectors-4/#typedef-ns-prefix>
    ///
    /// This method can return `Ok(None)` when parsing a selector like
    /// like `|foo` (with an empty [NamespacePrefix]), [since thats equivalent
    /// to not having a namespace specified at all](https://drafts.csswg.org/css-namespaces/#terminology)
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        let prefix = parser.parse_optional_value(|parser| match parser.next_token() {
            Some(Token::Ident(ident)) => Ok(NamespacePrefix::Ident(ident)),
            Some(Token::Delim('*')) => Ok(NamespacePrefix::Asterisk),
            _ => Err(ParseError),
        });

        parser.skip_whitespace();

        if let Some(Token::Delim('|')) = parser.next_token() {
            Ok(prefix)
        } else {
            Err(ParseError)
        }
    }
}

impl CSSValidateSelector for NamespacePrefix {
    fn is_valid(&self) -> bool {
        // We don't support *any* namespace prefixes
        // As per spec, we therefore treat them as invalid
        false
    }
}

#[cfg(test)]
mod tests {
    use super::NamespacePrefix;
    use crate::css::CSSParse;

    #[test]
    fn parse_ns_prefix() {
        assert_eq!(
            Option::<NamespacePrefix>::parse_from_str("foo |"),
            Ok(Some(NamespacePrefix::Ident("foo".into())))
        );
        assert_eq!(
            Option::<NamespacePrefix>::parse_from_str("* |"),
            Ok(Some(NamespacePrefix::Asterisk))
        );
        assert_eq!(Option::<NamespacePrefix>::parse_from_str("|"), Ok(None),);
    }
}
