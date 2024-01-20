//! <https://262.ecma-international.org/14.0/#sec-block>

use super::StatementListItem;
use crate::parser::{SyntaxError, Tokenizer};

/// <https://262.ecma-international.org/14.0/#prod-StatementList>
pub(crate) fn parse_statement_list<const YIELD: bool, const AWAIT: bool, const RETURN: bool>(
    tokenizer: &mut Tokenizer<'_>,
) -> Result<Vec<StatementListItem>, SyntaxError> {
    // FIXME: parse more than one statement here
    Ok(vec![StatementListItem::parse::<true, true, true>(
        tokenizer,
    )?])
}
