//! <https://262.ecma-international.org/14.0/#sec-unary-operators>

use crate::{
    bytecode::{self, CompileToBytecode},
    parser::{
        tokenization::{Punctuator, SkipLineTerminators, Token, Tokenizer},
        SyntaxError,
    },
};

use super::{Expression, UpdateExpression};

/// <https://262.ecma-international.org/14.0/#prod-UnaryExpression>
#[derive(Clone, Debug)]
pub enum UnaryExpression {
    Delete(Box<Expression>),
    Void(Box<Expression>),
    TypeOf(Box<Expression>),
    Plus(Box<Expression>),
    Minus(Box<Expression>),
    BitwiseNot(Box<Expression>),
    LogicalNot(Box<Expression>),
}

impl UnaryExpression {
    /// <https://262.ecma-international.org/14.0/#prod-UnaryExpression>
    pub fn parse<const YIELD: bool, const AWAIT: bool>(
        tokenizer: &mut Tokenizer<'_>,
    ) -> Result<Expression, SyntaxError> {
        let Some(next_token) = tokenizer.peek(0, SkipLineTerminators::Yes)? else {
            return Err(tokenizer.syntax_error());
        };

        let unary_expression = match next_token {
            Token::Punctuator(Punctuator::Plus) => {
                tokenizer.advance(1);
                let plus_expression = UnaryExpression::parse::<YIELD, AWAIT>(tokenizer)?;
                Self::Plus(Box::new(plus_expression))
            },
            Token::Punctuator(Punctuator::Minus) => {
                tokenizer.advance(1);
                let minus_expression = UnaryExpression::parse::<YIELD, AWAIT>(tokenizer)?;
                Self::Minus(Box::new(minus_expression))
            },
            Token::Punctuator(Punctuator::Tilde) => {
                tokenizer.advance(1);
                let bitwise_not_expression = UnaryExpression::parse::<YIELD, AWAIT>(tokenizer)?;
                Self::BitwiseNot(Box::new(bitwise_not_expression))
            },
            Token::Punctuator(Punctuator::ExclamationMark) => {
                tokenizer.advance(1);
                let logical_not_expression = UnaryExpression::parse::<YIELD, AWAIT>(tokenizer)?;
                Self::LogicalNot(Box::new(logical_not_expression))
            },
            Token::Identifier(ident) if ident == "delete" => {
                tokenizer.advance(1);
                let delete_expression = UnaryExpression::parse::<YIELD, AWAIT>(tokenizer)?;
                Self::Delete(Box::new(delete_expression))
            },
            Token::Identifier(ident) if ident == "void" => {
                tokenizer.advance(1);
                let void_expression = UnaryExpression::parse::<YIELD, AWAIT>(tokenizer)?;
                Self::Void(Box::new(void_expression))
            },
            Token::Identifier(ident) if ident == "typeof" => {
                tokenizer.advance(1);
                let typeof_expression = UnaryExpression::parse::<YIELD, AWAIT>(tokenizer)?;
                Self::TypeOf(Box::new(typeof_expression))
            },
            _ => return UpdateExpression::parse::<YIELD, AWAIT>(tokenizer),
        };

        Ok(unary_expression.into())
    }
}

impl CompileToBytecode for UnaryExpression {
    type Result = bytecode::Register;

    fn compile(&self, builder: &mut bytecode::ProgramBuilder) -> Self::Result {
        match self {
            Self::Delete(expression) => {
                _ = expression;
                todo!();
            },
            Self::Void(expression) => {
                _ = expression;
                todo!();
            },
            Self::TypeOf(expression) => {
                _ = expression;
                todo!();
            },
            Self::Plus(expression) => {
                // https://262.ecma-international.org/14.0/#sec-unary-plus-operator-runtime-semantics-evaluation

                // 1. Let expr be ? Evaluation of UnaryExpression.
                let result = expression.compile(builder);

                // 2. Return ? ToNumber(? GetValue(expr)).
                builder.get_current_block().to_number(result)
            },
            Self::Minus(expression) => {
                _ = expression;
                todo!();
            },
            Self::BitwiseNot(expression) => {
                _ = expression;
                todo!();
            },
            Self::LogicalNot(expression) => {
                _ = expression;
                todo!();
            },
        }
    }
}
