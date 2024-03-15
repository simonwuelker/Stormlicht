use crate::{
    bytecode::{self, CompileToBytecode},
    parser::{tokenizer::Punctuator, SyntaxError, Tokenizer},
};

use super::{
    block_statement::BlockStatement, if_statement::IfStatement, throw_statement::ThrowStatement,
    Declaration, WhileStatement,
};

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
    BlockStatement(BlockStatement),
    VariableStatement,
    EmptyStatement,
    ExpressionStatement,
    IfStatement(IfStatement),
    WhileStatement(WhileStatement),
    ContinueStatement,
    BreakStatement,
    RETURNStatement,
    WithStatement,
    LabelledStatement,
    ThrowStatement(ThrowStatement),
    TryStatement,
    DebuggerStatement,
}

impl Statement {
    /// <https://262.ecma-international.org/14.0/#prod-Statement>
    pub fn parse<const YIELD: bool, const AWAIT: bool, const RETURN: bool>(
        tokenizer: &mut Tokenizer<'_>,
    ) -> Result<Self, SyntaxError> {
        if let Ok(block_statement) =
            tokenizer.attempt(BlockStatement::parse::<YIELD, AWAIT, RETURN>)
        {
            Ok(Self::BlockStatement(block_statement))
        } else if let Ok(if_statement) =
            tokenizer.attempt(IfStatement::parse::<YIELD, AWAIT, RETURN>)
        {
            Ok(Self::IfStatement(if_statement))
        } else if let Ok(while_statement) =
            tokenizer.attempt(WhileStatement::parse::<YIELD, AWAIT, RETURN>)
        {
            Ok(Self::WhileStatement(while_statement))
        } else if tokenizer.attempt(parse_empty_statement).is_ok() {
            Ok(Self::EmptyStatement)
        } else if let Ok(throw_statement) = tokenizer.attempt(ThrowStatement::parse::<YIELD, AWAIT>)
        {
            Ok(Self::ThrowStatement(throw_statement))
        } else {
            Err(tokenizer.syntax_error())
        }
    }
}

impl CompileToBytecode for StatementListItem {
    fn compile(&self, builder: &mut bytecode::ProgramBuilder) {
        match self {
            Self::Statement(statement) => statement.compile(builder),
            Self::Declaration(declaration) => declaration.compile(builder),
        }
    }
}

impl CompileToBytecode for Statement {
    fn compile(&self, builder: &mut bytecode::ProgramBuilder) {
        match self {
            Self::BlockStatement(block_statement) => block_statement.compile(builder),
            Self::IfStatement(if_statement) => if_statement.compile(builder),
            Self::WhileStatement(while_statement) => while_statement.compile(builder),
            Self::EmptyStatement => {},
            Self::ThrowStatement(throw_statement) => throw_statement.compile(builder),
            _ => todo!(),
        }
    }
}

fn parse_empty_statement(tokenizer: &mut Tokenizer<'_>) -> Result<(), SyntaxError> {
    tokenizer.expect_punctuator(Punctuator::Semicolon)
}
