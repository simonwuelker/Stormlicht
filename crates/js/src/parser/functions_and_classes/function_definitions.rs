//! <https://262.ecma-international.org/14.0/#sec-function-definitions>

use crate::parser::{
    identifiers::parse_binding_identifier,
    statements_and_declarations::StatementListItem,
    tokenization::{Punctuator, SkipLineTerminators, Tokenizer},
    SyntaxError,
};

/// <https://262.ecma-international.org/14.0/#prod-FunctionDeclaration>
#[derive(Clone, Debug)]
pub struct FunctionDeclaration {
    identifier: String,
    body: Vec<StatementListItem>,
}

impl FunctionDeclaration {
    /// <https://262.ecma-international.org/14.0/#prod-FunctionDeclaration>
    pub fn parse<const YIELD: bool, const AWAIT: bool, const DEFAULT: bool>(
        tokenizer: &mut Tokenizer<'_>,
    ) -> Result<Self, SyntaxError> {
        tokenizer.expect_keyword("function")?;

        let identifier = parse_binding_identifier::<YIELD, AWAIT>(tokenizer)?;
        tokenizer.expect_punctuator(Punctuator::ParenthesisOpen)?;
        tokenizer.expect_punctuator(Punctuator::ParenthesisClose)?;
        tokenizer.expect_punctuator(Punctuator::CurlyBraceOpen)?;

        let mut body = vec![];
        while !tokenizer
            .peek(0, SkipLineTerminators::Yes)?
            .is_some_and(|t| t.is_punctuator(Punctuator::CurlyBraceClose))
        {
            let statement_list_item = StatementListItem::parse::<YIELD, AWAIT, true>(tokenizer)?;
            body.push(statement_list_item);
        }

        // Skip the closing curly brace
        tokenizer.advance(1);

        let function_declaration = Self { identifier, body };

        Ok(function_declaration)
    }

    #[must_use]
    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    #[must_use]
    pub fn body(&self) -> &[StatementListItem] {
        &self.body
    }
}
