//! <https://262.ecma-international.org/14.0/#sec-block>

use super::StatementListItem;
use crate::{
    bytecode::{self, CompileToBytecode},
    parser::{tokenizer::Punctuator, SyntaxError, Tokenizer},
};

/// <https://262.ecma-international.org/14.0/#prod-StatementList>
pub(crate) fn parse_statement_list<const YIELD: bool, const AWAIT: bool, const RETURN: bool>(
    tokenizer: &mut Tokenizer<'_>,
) -> Result<Vec<StatementListItem>, SyntaxError> {
    // FIXME: parse more than one statement here
    Ok(vec![StatementListItem::parse::<true, true, true>(
        tokenizer,
    )?])
}

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
        let statements = parse_statement_list::<YIELD, AWAIT, RETURN>(tokenizer)?;
        tokenizer.expect_punctuator(Punctuator::CurlyBraceClose)?;

        let block_statement = Self { statements };
        Ok(block_statement)
    }
}

impl CompileToBytecode for BlockStatement {
    fn compile(&self, builder: &mut bytecode::ProgramBuilder) {
        for statement in &self.statements {
            statement.compile(builder);
        }
    }
}
