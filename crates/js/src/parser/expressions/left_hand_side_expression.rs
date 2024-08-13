//! <https://262.ecma-international.org/14.0/#sec-left-hand-side-expressions>

use crate::parser::{
    tokenization::{Punctuator, SkipLineTerminators, Token, Tokenizer},
    SyntaxError,
};

use super::{
    call::parse_arguments, parse_primary_expression, CallExpression, Expression, MemberExpression,
};

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
        _ => {
            let member_expression = MemberExpression::parse::<YIELD, AWAIT>(tokenizer)?;

            let expression = match tokenizer.peek(0, SkipLineTerminators::Yes)? {
                Some(Token::Punctuator(Punctuator::ParenthesisOpen)) => {
                    // Parse call expression
                    let arguments = parse_arguments::<YIELD, AWAIT>(tokenizer)?;

                    CallExpression {
                        callable: Box::new(member_expression),
                        arguments,
                    }
                    .into()
                },
                _ => member_expression,
            };
            expression
        },
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
