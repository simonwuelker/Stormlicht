//! <https://262.ecma-international.org/14.0/#sec-ecmascript-language-statements-and-declarations>

mod block;
mod declaration;
mod if_statement;
mod statement;

pub(crate) use block::parse_statement_list;
pub(crate) use declaration::Declaration;
pub(crate) use statement::{Statement, StatementListItem};
