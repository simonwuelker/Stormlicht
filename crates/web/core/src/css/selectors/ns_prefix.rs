use crate::{
    css::{selectors::CSSValidateSelector, syntax::Token, CSSParse, ParseError, Parser},
    InternedString,
};

/// <https://drafts.csswg.org/selectors-4/#typedef-ns-prefix>
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NSPrefix {
    Ident(InternedString),
    Asterisk,
    Empty,
}

impl<'a> CSSParse<'a> for NSPrefix {
    // <https://drafts.csswg.org/selectors-4/#typedef-ns-prefix>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        let prefix = parser
            .parse_optional_value(|parser| match parser.next_token() {
                Some(Token::Ident(ident)) => Ok(NSPrefix::Ident(ident)),
                Some(Token::Delim('*')) => Ok(NSPrefix::Asterisk),
                _ => Err(ParseError),
            })
            .unwrap_or(NSPrefix::Empty);

        parser.skip_whitespace();

        if let Some(Token::Delim('|')) = parser.next_token() {
            Ok(prefix)
        } else {
            Err(ParseError)
        }
    }
}

impl CSSValidateSelector for NSPrefix {
    fn is_valid(&self) -> bool {
        // We don't support *any* namespace prefixes
        // As per spec, we therefore treat them as invalid
        false
    }
}

#[cfg(test)]
mod tests {
    use super::NSPrefix;
    use crate::css::CSSParse;

    #[test]
    fn parse_ns_prefix() {
        assert_eq!(
            NSPrefix::parse_from_str("foo |"),
            Ok(NSPrefix::Ident("foo".into()))
        );
        assert_eq!(NSPrefix::parse_from_str("* |"), Ok(NSPrefix::Asterisk));
        assert_eq!(NSPrefix::parse_from_str("|"), Ok(NSPrefix::Empty),);
    }
}
