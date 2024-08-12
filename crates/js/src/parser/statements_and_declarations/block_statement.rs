//! <https://262.ecma-international.org/14.0/#sec-block>

use super::StatementListItem;
use crate::{
    bytecode::{self, CompileToBytecode},
    parser::{
        tokenization::{Punctuator, SkipLineTerminators, Token, Tokenizer},
        SyntaxError,
    },
};

/// <https://262.ecma-international.org/14.0/#prod-BlockStatement>
#[derive(Clone, Debug)]
pub struct BlockStatement {
    pub statements: Vec<StatementListItem>,
}

impl BlockStatement {
    pub fn parse<const YIELD: bool, const AWAIT: bool, const RETURN: bool>(
        tokenizer: &mut Tokenizer<'_>,
    ) -> Result<Self, SyntaxError> {
        tokenizer.expect_punctuator(Punctuator::CurlyBraceOpen)?;

        let mut statements = vec![];
        while !matches!(
            tokenizer.peek(0, SkipLineTerminators::Yes)?,
            Some(Token::Punctuator(Punctuator::CurlyBraceClose))
        ) {
            let statement_list_item = StatementListItem::parse::<YIELD, AWAIT, RETURN>(tokenizer)?;
            statements.push(statement_list_item);
        }

        // Discard the closing brace
        tokenizer.advance(1);

        let block_statement = Self { statements };
        Ok(block_statement)
    }

    #[must_use]
    pub fn statement_list(&self) -> &[StatementListItem] {
        &self.statements
    }
}

impl CompileToBytecode for BlockStatement {
    fn compile(&self, builder: &mut bytecode::ProgramBuilder) {
        for statement in &self.statements {
            statement.compile(builder);
        }
    }
}
