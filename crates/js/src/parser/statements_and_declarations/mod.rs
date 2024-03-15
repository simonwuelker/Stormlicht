//! <https://262.ecma-international.org/14.0/#sec-ecmascript-language-statements-and-declarations>

mod block_statement;
mod declaration;
mod if_statement;
mod statement;
mod throw_statement;
mod while_statement;

pub(crate) use block_statement::parse_statement_list;
pub(crate) use declaration::Declaration;
pub(crate) use statement::{Statement, StatementListItem};
pub(crate) use while_statement::WhileStatement;
