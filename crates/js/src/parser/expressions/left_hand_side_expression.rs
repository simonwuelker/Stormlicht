//! <https://262.ecma-international.org/14.0/#sec-left-hand-side-expressions>

use crate::parser::{
    tokenization::{SkipLineTerminators, Token, Tokenizer},
    SyntaxError,
};

use super::{parse_primary_expression, Expression};

/// <https://262.ecma-international.org/14.0/#prod-LeftHandSideExpression>
#[derive(Clone, Debug)]
pub enum LeftHandSideExpression {
    NewExpression(NewExpression),
    CallExpression,
    OptionalExpression,
}

/// <https://262.ecma-international.org/14.0/#prod-NewExpression>
#[derive(Clone, Debug)]
pub struct NewExpression {
    /// The number of `new` keywords before the expression
    pub nest_level: usize,
    pub expression: Box<Expression>,
}

impl LeftHandSideExpression {
    /// <https://262.ecma-international.org/14.0/#prod-LeftHandSideExpression>
    pub fn parse<const YIELD: bool, const AWAIT: bool>(
        tokenizer: &mut Tokenizer<'_>,
    ) -> Result<Expression, SyntaxError> {
        let Some(next_token) = tokenizer.peek(0, SkipLineTerminators::Yes)? else {
            return Err(tokenizer.syntax_error());
        };

        let lhs_expression = match next_token {
            Token::Identifier(ident) if ident == "new" => {
                NewExpression::parse::<YIELD, AWAIT>(tokenizer)?
            },
            _ => parse_primary_expression::<YIELD, AWAIT>(tokenizer)?,
        };

        Ok(lhs_expression)
    }
}

impl NewExpression {
    /// <https://262.ecma-international.org/14.0/#prod-NewExpression>
    fn parse<const YIELD: bool, const AWAIT: bool>(
        tokenizer: &mut Tokenizer<'_>,
    ) -> Result<Expression, SyntaxError> {
        let mut nest_level = 0;

        while tokenizer
            .peek(0, SkipLineTerminators::Yes)?
            .is_some_and(|t| t.is_identifier("new"))
        {
            tokenizer.advance(1);
            nest_level += 1;
        }

        // FIXME: This should be a MemberExpression instead of a PrimaryExpression
        let member_expression = parse_primary_expression::<YIELD, AWAIT>(tokenizer)?;

        let new_expression = if nest_level == 0 {
            member_expression
        } else {
            Self {
                nest_level,
                expression: Box::new(member_expression),
            }
            .into()
        };

        Ok(new_expression)
    }
}
