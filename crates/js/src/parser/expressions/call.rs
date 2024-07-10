//! <https://262.ecma-international.org/14.0/#prod-CallExpression>
use crate::{
    bytecode::{self, CompileToBytecode},
    parser::{
        tokenization::{Punctuator, SkipLineTerminators, Token, Tokenizer},
        SyntaxError,
    },
};

use super::{AssignmentExpression, Expression};

/// <https://262.ecma-international.org/14.0/#prod-CallExpression>
#[derive(Clone, Debug)]
pub struct CallExpression {
    pub callable: Box<Expression>,
    pub arguments: Vec<Expression>,
}

/// <https://262.ecma-international.org/14.0/#prod-Arguments>
pub fn parse_arguments<const YIELD: bool, const AWAIT: bool>(
    tokenizer: &mut Tokenizer<'_>,
) -> Result<Vec<Expression>, SyntaxError> {
    tokenizer.expect_punctuator(Punctuator::ParenthesisOpen)?;

    let mut arguments = vec![];
    let mut next_token = tokenizer.peek(0, SkipLineTerminators::Yes)?;

    while !matches!(
        next_token,
        Some(Token::Punctuator(Punctuator::ParenthesisClose))
    ) {
        let argument = AssignmentExpression::parse::<true, YIELD, AWAIT>(tokenizer)?;
        arguments.push(argument);

        next_token = tokenizer.peek(0, SkipLineTerminators::Yes)?;

        // There may or may not be a comma - if there's not, then this is the last element
        if matches!(next_token, Some(Token::Punctuator(Punctuator::Comma))) {
            tokenizer.next(SkipLineTerminators::Yes)?;
            next_token = tokenizer.peek(0, SkipLineTerminators::Yes)?;
        } else {
            break;
        }
    }

    // Consume the final semicolon
    tokenizer.next(SkipLineTerminators::Yes)?;

    Ok(arguments)
}

impl CompileToBytecode for CallExpression {
    type Result = bytecode::Register;

    fn compile(&self, builder: &mut bytecode::ProgramBuilder) -> Self::Result {
        // https://262.ecma-international.org/14.0/#sec-function-calls-runtime-semantics-evaluation
        let callable = self.callable.compile(builder);
        let arguments = self
            .arguments
            .iter()
            .map(|arg| arg.compile(builder))
            .collect();

        builder.get_current_block().call(callable, arguments)
    }
}
