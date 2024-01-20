use crate::{
    bytecode::{self, CompileToBytecode},
    parser::{SyntaxError, Tokenizer},
};

use super::Declaration;

/// <https://262.ecma-international.org/14.0/#prod-StatementListItem>
#[derive(Clone, Debug)]
pub(crate) enum StatementListItem {
    Statement(Statement),
    Declaration(Declaration),
}

impl StatementListItem {
    /// <https://262.ecma-international.org/14.0/#prod-StatementListItem>
    pub(crate) fn parse<const YIELD: bool, const AWAIT: bool, const RETURN: bool>(
        tokenizer: &mut Tokenizer<'_>,
    ) -> Result<Self, SyntaxError> {
        let statement_list_item =
            if let Ok(statement) = tokenizer.attempt(Statement::parse::<YIELD, AWAIT, RETURN>) {
                Self::Statement(statement)
            } else if let Ok(declaration) = tokenizer.attempt(Declaration::parse::<YIELD, AWAIT>) {
                Self::Declaration(declaration)
            } else {
                return Err(tokenizer.syntax_error());
            };

        Ok(statement_list_item)
    }
}

/// <https://262.ecma-international.org/14.0/#prod-Statement>
#[derive(Clone, Debug)]
pub(crate) enum Statement {
    BlockStatement,
    VariableStatement,
    EmptyStatement,
    ExpressionStatement,
    IfStatement,
    BreakableStatement,
    ContinueStatement,
    BreakStatement,
    RETURNStatement,
    WithStatement,
    LabelledStatement,
    ThrowStatement,
    TryStatement,
    DebuggerStatement,
}

impl Statement {
    /// <https://262.ecma-international.org/14.0/#prod-Statement>
    fn parse<const YIELD: bool, const AWAIT: bool, const RETURN: bool>(
        tokenizer: &mut Tokenizer<'_>,
    ) -> Result<Self, SyntaxError> {
        // TODO
        _ = tokenizer;
        Err(tokenizer.syntax_error())
    }
}

impl CompileToBytecode for StatementListItem {
    fn compile(&self, builder: &mut bytecode::Builder) {
        match self {
            Self::Statement(statement) => statement.compile(builder),
            Self::Declaration(declaration) => declaration.compile(builder),
        }
    }
}

impl CompileToBytecode for Statement {
    fn compile(&self, builder: &mut bytecode::Builder) -> Self::Result {
        _ = builder;
        todo!()
    }
}
