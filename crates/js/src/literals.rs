use crate::{error::SyntaxError, Tokenizer};

/// <https://262.ecma-international.org/14.0/#prod-Literal>
#[derive(Clone, Debug)]
pub enum Literal {
    NullLiteral,
    BooleanLiteral(bool),
    NumericLiteral,
    StringLiteral(String),
}

impl Literal {
    pub fn parse(tokenizer: &mut Tokenizer<'_>) -> Result<Self, SyntaxError> {
        // FIXME: How should we propagate syntax errors here?
        if tokenizer.attempt(Tokenizer::consume_null_literal).is_ok() {
            Ok(Self::NullLiteral)
        } else if let Ok(bool_literal) = tokenizer.attempt(Tokenizer::consume_boolean_literal) {
            Ok(Self::BooleanLiteral(bool_literal))
        } else if let Ok(string_literal) = tokenizer.attempt(Tokenizer::consume_string_literal) {
            Ok(Self::StringLiteral(string_literal))
        } else {
            Err(SyntaxError)
        }
    }
}
