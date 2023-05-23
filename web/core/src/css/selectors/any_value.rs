use crate::css::{syntax::Token, CSSParse, ParseError, Parser};

/// <https://w3c.github.io/csswg-drafts/css-syntax-3/#typedef-any-value>
#[derive(Clone, Debug, PartialEq)]
pub struct AnyValue<'a>(pub Vec<Token<'a>>);

impl<'a> CSSParse<'a> for AnyValue<'a> {
    // <https://w3c.github.io/csswg-drafts/css-syntax-3/#typedef-any-value>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        let mut values = vec![];

        let mut parenthesis_balance = 0;
        let mut bracket_balance = 0;
        let mut curly_brace_balance = 0;
        let mut state_before_ending_token;

        loop {
            state_before_ending_token = parser.state();

            match parser.next_token() {
                None | Some(Token::BadString(_)) | Some(Token::BadURI(_)) => break,
                Some(Token::ParenthesisOpen) => parenthesis_balance += 1,
                Some(Token::BracketOpen) => bracket_balance += 1,
                Some(Token::CurlyBraceOpen) => curly_brace_balance += 1,
                Some(Token::ParenthesisClose) => {
                    if parenthesis_balance == 0 {
                        break;
                    } else {
                        parenthesis_balance -= 1;
                    }
                },
                Some(Token::BracketClose) => {
                    if bracket_balance == 0 {
                        break;
                    } else {
                        bracket_balance -= 1;
                    }
                },
                Some(Token::CurlyBraceClose) => {
                    if curly_brace_balance == 0 {
                        break;
                    } else {
                        curly_brace_balance -= 1;
                    }
                },
                Some(other) => values.push(other),
            }
        }
        parser.set_state(state_before_ending_token);

        if values.is_empty() {
            return Err(ParseError);
        }

        Ok(Self(values))
    }
}
