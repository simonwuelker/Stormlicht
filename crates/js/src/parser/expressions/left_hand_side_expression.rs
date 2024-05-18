//! <https://262.ecma-international.org/14.0/#sec-left-hand-side-expressions>

use crate::{
    bytecode::{self, CompileToBytecode},
    parser::{
        tokenization::{SkipLineTerminators, Token, Tokenizer},
        SyntaxError,
    },
};

use super::{parse_primary_expression, Expression, MemberExpression};

/// <https://262.ecma-international.org/14.0/#prod-NewExpression>
#[derive(Clone, Debug)]
pub struct NewExpression {
    /// The number of `new` keywords before the expression
    pub nest_level: usize,
    pub expression: Box<Expression>,
}

/// <https://262.ecma-international.org/14.0/#prod-LeftHandSideExpression>
pub fn parse_lefthandside_expression<const YIELD: bool, const AWAIT: bool>(
    tokenizer: &mut Tokenizer<'_>,
) -> Result<Expression, SyntaxError> {
    let Some(next_token) = tokenizer.peek(0, SkipLineTerminators::Yes)? else {
        return Err(tokenizer.syntax_error("expected more tokens"));
    };

    let lhs_expression = match next_token {
        Token::Identifier(ident) if ident == "new" => {
            NewExpression::parse::<YIELD, AWAIT>(tokenizer)?
        },
        _ => MemberExpression::parse::<YIELD, AWAIT>(tokenizer)?,
    };

    Ok(lhs_expression)
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

impl CompileToBytecode for NewExpression {
    type Result = bytecode::Register;

    fn compile(&self, builder: &mut bytecode::ProgramBuilder) -> Self::Result {
        _ = builder;
        _ = self.nest_level;
        _ = self.expression;
        todo!("compile NewExpression")
    }
}
