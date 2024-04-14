//! <https://262.ecma-international.org/14.0/#sec-while-statement>

use crate::{
    bytecode::{self, CompileToBytecode},
    parser::{
        expressions::Expression,
        tokenization::{Punctuator, Tokenizer},
        SyntaxError,
    },
};

use super::Statement;

/// <https://262.ecma-international.org/14.0/#sec-while-statement>
#[derive(Clone, Debug)]
pub struct WhileStatement {
    pub loop_condition: Expression,
    pub body: Box<Statement>,
}

impl WhileStatement {
    /// <https://262.ecma-international.org/14.0/#sec-while-statement>
    pub fn parse<const YIELD: bool, const AWAIT: bool, const RETURN: bool>(
        tokenizer: &mut Tokenizer<'_>,
    ) -> Result<Self, SyntaxError> {
        tokenizer.expect_keyword("while")?;
        tokenizer.expect_punctuator(Punctuator::ParenthesisOpen)?;
        let loop_condition = Expression::parse::<true, YIELD, AWAIT>(tokenizer)?;
        tokenizer.expect_punctuator(Punctuator::ParenthesisClose)?;
        let body = Statement::parse::<YIELD, AWAIT, RETURN>(tokenizer)?;

        let while_statement = Self {
            loop_condition,
            body: Box::new(body),
        };

        Ok(while_statement)
    }
}

impl CompileToBytecode for WhileStatement {
    fn compile(&self, builder: &mut bytecode::ProgramBuilder) -> Self::Result {
        _ = builder;
        _ = self.loop_condition;
        _ = self.body;
        todo!()
    }
}
