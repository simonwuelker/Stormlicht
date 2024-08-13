//! <https://262.ecma-international.org/14.0/#sec-update-expressions>

use crate::parser::{
    tokenization::{Punctuator, SkipLineTerminators, Token, Tokenizer},
    SyntaxError,
};

use super::{left_hand_side_expression::parse_lefthandside_expression, Expression};

/// <https://262.ecma-international.org/14.0/#prod-UpdateExpression>
#[derive(Clone, Debug)]
pub enum UpdateExpression {
    /// `++foo`
    PreIncrement(Box<Expression>),

    /// `foo++`
    PostIncrement(Box<Expression>),

    /// `--foo`
    PreDecrement(Box<Expression>),

    /// `foo--`
    PostDecrement(Box<Expression>),
}

impl UpdateExpression {
    pub fn parse<const YIELD: bool, const AWAIT: bool>(
        tokenizer: &mut Tokenizer<'_>,
    ) -> Result<Expression, SyntaxError> {
        let Some(next_token) = tokenizer.peek(0, SkipLineTerminators::Yes)? else {
            return Err(tokenizer.syntax_error("expected more tokens"));
        };

        let update_expression = match next_token {
            Token::Punctuator(Punctuator::DoublePlus) => {
                tokenizer.advance(1);
                let lhs_expression = parse_lefthandside_expression::<YIELD, AWAIT>(tokenizer)?;
                Self::PreIncrement(Box::new(lhs_expression))
            },
            Token::Punctuator(Punctuator::DoubleMinus) => {
                tokenizer.advance(1);
                let lhs_expression = parse_lefthandside_expression::<YIELD, AWAIT>(tokenizer)?;
                Self::PreDecrement(Box::new(lhs_expression))
            },
            _ => {
                let lhs_expression = parse_lefthandside_expression::<YIELD, AWAIT>(tokenizer)?;

                match tokenizer.peek(0, SkipLineTerminators::No)? {
                    Some(Token::Punctuator(Punctuator::DoublePlus)) => {
                        Self::PostIncrement(Box::new(lhs_expression))
                    },
                    Some(Token::Punctuator(Punctuator::DoubleMinus)) => {
                        Self::PostDecrement(Box::new(lhs_expression))
                    },
                    _ => return Ok(lhs_expression),
                }
            },
        };

        Ok(update_expression.into())
    }
}
