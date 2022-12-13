pub mod error;
pub mod parser_combinators;

use error::SyntaxError;
use parser_combinators::{Literal, Many, SourceText};
use std::io::Cursor;

/// Contains information about a script being evaluated
pub struct ScriptRecord {
    code: ParseNode,
}

struct ParseNode;

// JS Grammar defintion
// const STATEMENT_LIST_ITEM: Literal = Literal::new(&b"test"[..]);
// const STATEMENT_LIST: Many = Many::new(STATEMENT_LIST_ITEM);

pub fn parse_script(source_text: SourceText) -> Result<ScriptRecord, Vec<SyntaxError>> {
    let buffer = Cursor::new(source_text);
    _ = buffer;

    Ok(ScriptRecord { code: ParseNode })
}
