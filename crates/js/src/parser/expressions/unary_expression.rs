//! <https://262.ecma-international.org/14.0/#sec-unary-operators>

use crate::parser::{
    tokenization::{Punctuator, SkipLineTerminators, Token, Tokenizer},
    SyntaxError,
};

use super::{Expression, UpdateExpression};

/// <https://262.ecma-international.org/14.0/#prod-UnaryExpression>
#[derive(Clone, Debug)]
pub struct UnaryExpression {
    operator: UnaryOperator,
    expression: Box<Expression>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnaryOperator {
    Delete,
    Void,
    TypeOf,
    Plus,
    Minus,
    BitwiseNot,
    LogicalNot,
}

impl UnaryExpression {
    /// <https://262.ecma-international.org/14.0/#prod-UnaryExpression>
    pub fn parse<const YIELD: bool, const AWAIT: bool>(
        tokenizer: &mut Tokenizer<'_>,
    ) -> Result<Expression, SyntaxError> {
        let Some(next_token) = tokenizer.peek(0, SkipLineTerminators::Yes)? else {
            return Err(tokenizer.syntax_error("expected more tokens"));
        };

        let operator = match next_token {
            Token::Punctuator(Punctuator::Plus) => {
                tokenizer.advance(1);
                UnaryOperator::Plus
            },
            Token::Punctuator(Punctuator::Minus) => {
                tokenizer.advance(1);
                UnaryOperator::Minus
            },
            Token::Punctuator(Punctuator::Tilde) => {
                tokenizer.advance(1);
                UnaryOperator::BitwiseNot
            },
            Token::Punctuator(Punctuator::ExclamationMark) => {
                tokenizer.advance(1);
                UnaryOperator::LogicalNot
            },
            Token::Identifier(ident) if ident == "delete" => {
                tokenizer.advance(1);
                UnaryOperator::Delete
            },
            Token::Identifier(ident) if ident == "void" => {
                tokenizer.advance(1);
                UnaryOperator::Void
            },
            Token::Identifier(ident) if ident == "typeof" => {
                tokenizer.advance(1);
                UnaryOperator::TypeOf
            },
            _ => return UpdateExpression::parse::<YIELD, AWAIT>(tokenizer),
        };
        let expression = UnaryExpression::parse::<YIELD, AWAIT>(tokenizer)?;

        let unary_expression = Self {
            operator,
            expression: Box::new(expression),
        };

        Ok(unary_expression.into())
    }
}
