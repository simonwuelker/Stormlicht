//! <https://262.ecma-international.org/14.0/#sec-throw-statement>

use crate::parser::{expressions::Expression, tokenization::Tokenizer, SyntaxError};

/// <https://262.ecma-international.org/14.0/#sec-throw-statement>
#[derive(Clone, Debug)]
pub struct ThrowStatement {
    expression: Expression,
}

impl ThrowStatement {
    /// <https://262.ecma-international.org/14.0/#prod-ThrowStatement>
    pub fn parse<const YIELD: bool, const AWAIT: bool>(
        tokenizer: &mut Tokenizer<'_>,
    ) -> Result<Self, SyntaxError> {
        tokenizer.expect_keyword("throw")?;

        tokenizer.expect_no_line_terminator()?;
        let expression = Expression::parse::<true, YIELD, AWAIT>(tokenizer)?;

        let throw_statement = Self { expression };

        tokenizer.expect_semicolon()?;

        Ok(throw_statement)
    }
}
