//! <https://262.ecma-international.org/14.0/#sec-update-expressions>

use crate::{
    bytecode::{self, CompileToBytecode},
    parser::{
        tokenization::{Punctuator, SkipLineTerminators, Token, Tokenizer},
        SyntaxError,
    },
};

use super::{Expression, LeftHandSideExpression};

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
            return Err(tokenizer.syntax_error());
        };

        let update_expression = match next_token {
            Token::Punctuator(Punctuator::DoublePlus) => {
                tokenizer.advance(1);
                let lhs_expression = LeftHandSideExpression::parse::<YIELD, AWAIT>(tokenizer)?;
                Self::PreIncrement(Box::new(lhs_expression))
            },
            Token::Punctuator(Punctuator::DoubleMinus) => {
                tokenizer.advance(1);
                let lhs_expression = LeftHandSideExpression::parse::<YIELD, AWAIT>(tokenizer)?;
                Self::PreDecrement(Box::new(lhs_expression))
            },
            _ => {
                let lhs_expression = LeftHandSideExpression::parse::<YIELD, AWAIT>(tokenizer)?;

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

impl CompileToBytecode for UpdateExpression {
    type Result = bytecode::Register;

    fn compile(&self, builder: &mut bytecode::ProgramBuilder) -> Self::Result {
        let current_block = builder.get_current_block();

        _ = current_block;
        todo!()
    }
}
