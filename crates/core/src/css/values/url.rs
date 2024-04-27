//! <https://drafts.csswg.org/css-values-3/#urls>

use crate::{
    css::{syntax::Token, CSSParse, ParseError, Parser},
    static_interned, InternedString,
};

#[derive(Clone, Debug)]
pub struct Url {
    value: InternedString,
}

impl Url {
    #[must_use]
    pub fn value(&self) -> InternedString {
        self.value
    }
}

impl<'a> CSSParse<'a> for Url {
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        let url = match parser.next_token() {
            Some(Token::Uri(url)) => url,
            Some(Token::Function(f)) if f == static_interned!("url") => {
                let Some(Token::String(url)) = parser.next_token() else {
                    return Err(ParseError);
                };
                parser.expect_token(Token::ParenthesisClose)?;

                url
            },
            _ => return Err(ParseError),
        };
        let url = Self { value: url };

        Ok(url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_url_function() {
        let expected_url = "https://www.google.com";
        let text = format!("url(\"{expected_url}\")");

        let parsed_url = Url::parse_from_str(&text).unwrap();
        assert_eq!(parsed_url.value().to_string(), expected_url);
    }

    #[test]
    fn parse_unquoted_url() {
        let expected_url = "https://www.google.com";
        let text = format!("url({expected_url})");

        let parsed_url = Url::parse_from_str(&text).unwrap();
        assert_eq!(parsed_url.value().to_string(), expected_url);
    }
}
