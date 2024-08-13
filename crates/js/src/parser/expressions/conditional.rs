//! <https://262.ecma-international.org/14.0/#sec-conditional-operator>

use crate::parser::{
    tokenization::{Punctuator, SkipLineTerminators, Token, Tokenizer},
    SyntaxError,
};

use super::{short_circuit::parse_short_circuit_expression, AssignmentExpression, Expression};

/// <https://262.ecma-international.org/14.0/#prod-ConditionalExpression>
#[derive(Clone, Debug)]
pub struct ConditionalExpression {
    condition: Box<Expression>,
    true_case: Box<Expression>,
    false_case: Box<Expression>,
}

impl ConditionalExpression {
    /// <https://262.ecma-international.org/14.0/#prod-ConditionalExpression>
    pub fn parse<const IN: bool, const YIELD: bool, const AWAIT: bool>(
        tokenizer: &mut Tokenizer<'_>,
    ) -> Result<Expression, SyntaxError> {
        let condition = parse_short_circuit_expression::<IN, YIELD, AWAIT>(tokenizer)?;

        if let Some(Token::Punctuator(Punctuator::QuestionMark)) =
            tokenizer.peek(0, SkipLineTerminators::Yes)?
        {
            tokenizer.advance(1);

            let true_case = AssignmentExpression::parse::<IN, YIELD, AWAIT>(tokenizer)?;

            tokenizer.expect_punctuator(Punctuator::Colon)?;

            let false_case = AssignmentExpression::parse::<IN, YIELD, AWAIT>(tokenizer)?;

            let conditional_expression = ConditionalExpression {
                condition: Box::new(condition),
                true_case: Box::new(true_case),
                false_case: Box::new(false_case),
            };

            Ok(conditional_expression.into())
        } else {
            Ok(condition)
        }
    }
}
