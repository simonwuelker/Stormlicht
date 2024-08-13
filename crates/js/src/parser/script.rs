use super::{
    statements_and_declarations::StatementListItem,
    tokenization::{SkipLineTerminators, Tokenizer},
    SyntaxError,
};

/// <https://262.ecma-international.org/14.0/#prod-ScriptBody>
#[derive(Clone, Debug)]
pub struct Script(Vec<StatementListItem>);

impl Script {
    /// <https://262.ecma-international.org/14.0/#prod-ScriptBody>
    pub fn parse(tokenizer: &mut Tokenizer<'_>) -> Result<Self, SyntaxError> {
        let mut statement_list_items = vec![];
        while tokenizer.peek(0, SkipLineTerminators::Yes)?.is_some() {
            let statement_list_item = StatementListItem::parse::<false, false, false>(tokenizer)?;
            statement_list_items.push(statement_list_item);
        }

        Ok(Self(statement_list_items))
    }

    #[must_use]
    pub fn statement_list(&self) -> &[StatementListItem] {
        &self.0
    }
}
