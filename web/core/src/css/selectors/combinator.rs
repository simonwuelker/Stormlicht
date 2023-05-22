use super::CSSValidateSelector;
use crate::css::{
    parser::{CSSParse, ParseError, Parser},
    tokenizer::Token,
};

/// <https://drafts.csswg.org/selectors-4/#typedef-combinator>
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Combinator {
    /// ` `
    ///
    /// # Specification
    /// <https://drafts.csswg.org/selectors-4/#descendant-combinators>
    #[default]
    Descendant,

    /// `>`
    ///
    /// # Specification
    /// <https://drafts.csswg.org/selectors-4/#child-combinators>
    Child,

    /// `+`
    ///
    /// # Specification
    /// <https://drafts.csswg.org/selectors-4/#adjacent-sibling-combinators>
    NextSibling,

    /// `~`
    ///
    /// # Specification
    /// <https://drafts.csswg.org/selectors-4/#general-sibling-combinators>
    SubsequentSibling,

    /// `||`
    ///
    /// # Specification
    /// <https://drafts.csswg.org/selectors-4/#the-column-combinator>
    Column,
}

impl<'a> CSSParse<'a> for Combinator {
    // <https://drafts.csswg.org/selectors-4/#typedef-combinator>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        match parser.next_token() {
            Some(Token::Delim('>')) => Ok(Combinator::Child),
            Some(Token::Delim('+')) => Ok(Combinator::NextSibling),
            Some(Token::Delim('~')) => Ok(Combinator::SubsequentSibling),
            Some(Token::Delim('|')) => {
                if matches!(parser.next_token(), Some(Token::Delim('|'))) {
                    Ok(Combinator::Column)
                } else {
                    Err(ParseError)
                }
            },
            _ => Err(ParseError),
        }
    }
}

impl CSSValidateSelector for Combinator {
    fn is_valid(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::Combinator;
    use crate::css::parser::CSSParse;

    #[test]
    fn parse_combinator() {
        assert_eq!(Combinator::parse_from_str(">"), Ok(Combinator::Child));
        assert_eq!(Combinator::parse_from_str("+"), Ok(Combinator::NextSibling));
        assert_eq!(
            Combinator::parse_from_str("~"),
            Ok(Combinator::SubsequentSibling)
        );
        assert_eq!(Combinator::parse_from_str("||"), Ok(Combinator::Column));
    }
}
