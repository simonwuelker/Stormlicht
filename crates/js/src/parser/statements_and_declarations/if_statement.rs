//! <https://262.ecma-international.org/14.0/#sec-if-statement>

use crate::{
    bytecode::{self, CompileToBytecode},
    parser::{tokenizer::Punctuator, Expression, SyntaxError, Tokenizer},
};

use super::Statement;

/// <https://262.ecma-international.org/14.0/#prod-IfStatement>
#[derive(Clone, Debug)]
pub struct IfStatement {
    pub condition: Expression,
    pub if_branch: Box<Statement>,
    pub else_branch: Option<Box<Statement>>,
}

impl IfStatement {
    /// <https://262.ecma-international.org/14.0/#prod-IfStatement>
    pub fn parse<const YIELD: bool, const AWAIT: bool, const RETURN: bool>(
        tokenizer: &mut Tokenizer<'_>,
    ) -> Result<Self, SyntaxError> {
        tokenizer.expect_keyword("if")?;
        tokenizer.expect_punctuator(Punctuator::ParenthesisOpen)?;
        let condition = Expression::parse::<true, YIELD, AWAIT>(tokenizer)?;
        tokenizer.expect_punctuator(Punctuator::ParenthesisClose)?;

        let if_branch = Statement::parse::<YIELD, AWAIT, RETURN>(tokenizer)?;

        let else_branch = tokenizer
            .attempt(parse_else_branch::<YIELD, AWAIT, RETURN>)
            .ok();

        let if_statement = Self {
            condition,
            if_branch: Box::new(if_branch),
            else_branch: else_branch.map(Box::new),
        };

        Ok(if_statement)
    }
}

fn parse_else_branch<const YIELD: bool, const AWAIT: bool, const RETURN: bool>(
    tokenizer: &mut Tokenizer<'_>,
) -> Result<Statement, SyntaxError> {
    tokenizer.expect_keyword("else")?;
    let else_branch = Statement::parse::<YIELD, AWAIT, RETURN>(tokenizer)?;

    Ok(else_branch)
}

impl CompileToBytecode for IfStatement {
    fn compile(&self, builder: &mut bytecode::Builder) -> Self::Result {
        _ = builder;
        todo!()
    }
}
