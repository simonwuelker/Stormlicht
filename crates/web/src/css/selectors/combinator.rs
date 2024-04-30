use std::fmt;

use crate::css::{selectors::CSSValidateSelector, syntax::Token, CSSParse, ParseError, Parser};

/// <https://drafts.csswg.org/selectors-4/#combinators>
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

impl Combinator {
    pub fn is_descendant(&self) -> bool {
        *self == Self::Descendant
    }

    #[must_use]
    pub fn next_combinator(parser: &mut Parser<'_>) -> Result<Option<Self>, ParseError> {
        let has_whitespace = parser.peek_token(0).is_some_and(Token::is_whitespace);

        if has_whitespace {
            _ = parser.next_token();
        }

        let combinator = match parser.peek_token(0) {
            Some(Token::Delim('>')) => {
                _ = parser.next_token();
                Combinator::Child
            },
            Some(Token::Delim('+')) => {
                _ = parser.next_token();
                Combinator::NextSibling
            },
            Some(Token::Delim('~')) => {
                _ = parser.next_token();
                Combinator::SubsequentSibling
            },
            Some(Token::Delim('|')) => {
                // This is either the beginning of a column combinator or a descendant combinator with a type selector afterwards
                if matches!(parser.peek_token(1), Some(Token::Delim('|'))) {
                    _ = parser.next_token();
                    _ = parser.next_token();

                    Combinator::Column
                } else {
                    // not a combinator
                    if has_whitespace {
                        Combinator::Descendant
                    } else {
                        return Err(ParseError);
                    }
                }
            },
            // These tokens are valid starts of simple selectors
            Some(
                Token::Delim('*' | '.') | Token::Ident(_) | Token::Hash(..) | Token::BracketOpen,
            ) => {
                // There is no combinator between these two simple selectors.
                // There *must* be at least one whitespace (descendant combinator)
                if !has_whitespace {
                    return Err(ParseError);
                }

                Combinator::Descendant
            },
            _ => {
                // There is no combinator here
                return Ok(None);
            },
        };

        Ok(Some(combinator))
    }
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
        // We don't support *any* combinators
        // As per spec, we therefore treat them as invalid
        false
    }
}

#[cfg(test)]
mod tests {
    use super::Combinator;
    use crate::css::CSSParse;

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
