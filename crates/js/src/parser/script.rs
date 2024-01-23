use crate::bytecode::{self, CompileToBytecode};

use super::{
    statements_and_declarations::{parse_statement_list, StatementListItem},
    SyntaxError, Tokenizer,
};

/// <https://262.ecma-international.org/14.0/#prod-ScriptBody>
#[derive(Clone, Debug)]
pub struct Script(Vec<StatementListItem>);

impl Script {
    pub fn parse(tokenizer: &mut Tokenizer<'_>) -> Result<Self, SyntaxError> {
        let statements = parse_statement_list::<true, true, true>(tokenizer)?;

        Ok(Self(statements))
    }
}

impl CompileToBytecode for Script {
    fn compile(&self, builder: &mut bytecode::ProgramBuilder) {
        for statement in &self.0 {
            statement.compile(builder);
        }
    }
}
