use crate::{Number, Value};

use super::{SyntaxError, Tokenizer};

/// <https://262.ecma-international.org/14.0/#prod-Literal>
#[derive(Clone, Debug)]
pub enum Literal {
    NullLiteral,
    BooleanLiteral(bool),
    NumericLiteral(Number),
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
        } else if let Ok(numeric_literal) = tokenizer.attempt(Tokenizer::consume_numeric_literal) {
            Ok(Self::NumericLiteral(numeric_literal))
        } else {
            Err(tokenizer.syntax_error())
        }
    }
}

impl From<Literal> for Value {
    fn from(value: Literal) -> Self {
        match value {
            Literal::NullLiteral => Self::Null,
            Literal::BooleanLiteral(bool) => Self::Boolean(bool),
            Literal::NumericLiteral(number) => Self::Number(number),
            Literal::StringLiteral(s) => Self::String(s),
        }
    }
}
