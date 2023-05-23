use std::borrow::Cow;

use super::{CSSValidateSelector, NSPrefix};
use crate::css::{syntax::Token, CSSParse, ParseError, Parser};

/// <https://drafts.csswg.org/selectors-4/#typedef-wq-name>
#[derive(Clone, Debug, PartialEq)]
pub struct WQName<'a> {
    pub prefix: Option<NSPrefix<'a>>,
    pub ident: Cow<'a, str>,
}

impl<'a> CSSParse<'a> for WQName<'a> {
    // <https://drafts.csswg.org/selectors-4/#typedef-wq-name>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        let prefix = parser.parse_optional_value(NSPrefix::parse);

        parser.skip_whitespace();

        if let Some(Token::Ident(ident)) = parser.next_token() {
            Ok(WQName { prefix, ident })
        } else {
            Err(ParseError)
        }
    }
}

impl<'a> CSSValidateSelector for WQName<'a> {
    fn is_valid(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::WQName;
    use crate::css::{selectors::NSPrefix, CSSParse};

    #[test]
    fn parse_wq_name() {
        assert_eq!(
            WQName::parse_from_str("foo | bar"),
            Ok(WQName {
                prefix: Some(NSPrefix::Ident("foo".into())),
                ident: "bar".into()
            })
        );

        assert_eq!(
            WQName::parse_from_str("bar"),
            Ok(WQName {
                prefix: None,
                ident: "bar".into()
            })
        );
    }
}
