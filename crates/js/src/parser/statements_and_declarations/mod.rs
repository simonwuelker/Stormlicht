//! <https://262.ecma-international.org/14.0/#sec-ecmascript-language-statements-and-declarations>

pub mod block_statement;
pub mod declaration;
pub mod if_statement;
pub mod statement;
pub mod throw_statement;
pub mod while_statement;

pub use declaration::{Declaration, LexicalBinding, LexicalDeclaration};
pub use if_statement::IfStatement;
pub use statement::{Statement, StatementListItem};
pub use while_statement::WhileStatement;
