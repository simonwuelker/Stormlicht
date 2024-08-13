//! <https://262.ecma-international.org/14.0/#sec-if-statement>

use crate::parser::{
    expressions::Expression,
    tokenization::{Punctuator, SkipLineTerminators, Token, Tokenizer},
    SyntaxError,
};

use super::Statement;

/// <https://262.ecma-international.org/14.0/#prod-IfStatement>
#[derive(Clone, Debug)]
pub struct IfStatement {
    condition: Expression,
    if_branch: Box<Statement>,
    else_branch: Option<Box<Statement>>,
}

impl IfStatement {
    #[must_use]
    pub fn condition(&self) -> &Expression {
        &self.condition
    }

    #[must_use]
    pub fn if_branch(&self) -> &Statement {
        &self.if_branch
    }

    #[must_use]
    pub fn else_branch(&self) -> Option<&Statement> {
        self.else_branch.as_deref()
    }

    /// <https://262.ecma-international.org/14.0/#prod-IfStatement>
    pub fn parse<const YIELD: bool, const AWAIT: bool, const RETURN: bool>(
        tokenizer: &mut Tokenizer<'_>,
    ) -> Result<Self, SyntaxError> {
        tokenizer.expect_keyword("if")?;
        tokenizer.expect_punctuator(Punctuator::ParenthesisOpen)?;
        let condition = Expression::parse::<true, YIELD, AWAIT>(tokenizer)?;
        tokenizer.expect_punctuator(Punctuator::ParenthesisClose)?;

        let if_branch = Statement::parse::<YIELD, AWAIT, RETURN>(tokenizer)?;

        let else_branch = match tokenizer.peek(0, SkipLineTerminators::Yes)? {
            Some(Token::Identifier(ident)) if ident == "else" => {
                tokenizer.advance(1);

                // There is an else branch following
                let else_branch = Statement::parse::<YIELD, AWAIT, RETURN>(tokenizer)?;
                Some(else_branch)
            },
            _ => None,
        };

        let if_statement = Self {
            condition,
            if_branch: Box::new(if_branch),
            else_branch: else_branch.map(Box::new),
        };

        Ok(if_statement)
    }
}
