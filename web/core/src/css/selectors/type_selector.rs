use super::{CSSValidateSelector, NSPrefix, WQName};
use crate::css::{
    parser::{CSSParse, ParseError, Parser},
    tokenizer::Token,
};

/// <https://drafts.csswg.org/selectors-4/#typedef-type-selector>
#[derive(Clone, Debug, PartialEq)]
pub enum TypeSelector<'a> {
    NSPrefix(Option<NSPrefix<'a>>),
    WQName(WQName<'a>),
}

impl<'a> CSSParse<'a> for TypeSelector<'a> {
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
            return Ok(TypeSelector::NSPrefix(ns_prefix));
        } else {
            Err(ParseError)
        }
    }
}

impl<'a> CSSValidateSelector for TypeSelector<'a> {}

#[cfg(test)]
mod tests {
    use super::TypeSelector;
    use crate::css::{
        parser::CSSParse,
        selectors::{NSPrefix, WQName},
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
