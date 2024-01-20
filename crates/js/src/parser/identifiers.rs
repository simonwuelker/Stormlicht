//! <https://262.ecma-international.org/14.0/#sec-identifiers>

use super::{tokenizer::GoalSymbol, SyntaxError, Tokenizer};

const RESERVED_WORDS: [&str; 37] = [
    "await",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "debugger",
    "default",
    "delete",
    "do",
    "else",
    "enum",
    "export",
    "extends",
    "false",
    "finally",
    "for",
    "function",
    "if",
    "import",
    "instanceof",
    "new",
    "null",
    "return",
    "super",
    "switch",
    "this",
    "throw",
    "true",
    "try",
    "typeof",
    "var",
    "void",
    "while",
    "with",
    "yield",
];

/// <https://262.ecma-international.org/14.0/#prod-BindingIdentifier>
pub(crate) fn parse_binding_identifier<const YIELD: bool, const AWAIT: bool>(
    tokenizer: &mut Tokenizer<'_>,
) -> Result<String, SyntaxError> {
    let binding_identifier = if let Ok(identifier) = tokenizer.attempt(Identifier::parse) {
        if tokenizer.is_strict() && matches!(identifier.0.as_str(), "arguments" | "eval") {
            return Err(tokenizer.syntax_error());
        }

        identifier.0
    } else {
        let identifier_name = tokenizer.consume_identifier()?;

        if !YIELD && identifier_name.as_str() == "yield" {
            identifier_name
        } else if !AWAIT && identifier_name.as_str() == "await" {
            identifier_name
        } else {
            return Err(tokenizer.syntax_error());
        }
    };

    Ok(binding_identifier)
}

/// <https://262.ecma-international.org/14.0/#prod-Identifier>
#[derive(Clone, Debug)]
pub struct Identifier(String);

const DISALLOWED_IDENTIFIERS_IN_STRICT_MODE: [&str; 9] = [
    "implements",
    "interface",
    "let",
    "package",
    "private",
    "protected",
    "public",
    "static",
    "yield",
];

impl Identifier {
    /// <https://262.ecma-international.org/14.0/#prod-Identifier>
    pub(crate) fn parse(tokenizer: &mut Tokenizer<'_>) -> Result<Self, SyntaxError> {
        let identifier_name = tokenizer.consume_identifier()?;
        if RESERVED_WORDS.contains(&identifier_name.as_str()) {
            return Err(tokenizer.syntax_error());
        }

        if tokenizer.is_strict()
            && DISALLOWED_IDENTIFIERS_IN_STRICT_MODE.contains(&identifier_name.as_str())
        {
            return Err(tokenizer.syntax_error());
        }

        if tokenizer.goal_symbol() == GoalSymbol::Module && identifier_name.as_str() == "await" {
            return Err(tokenizer.syntax_error());
        }

        Ok(Self(identifier_name))
    }
}

/// <https://262.ecma-international.org/14.0/#prod-IdentifierReference>
pub(crate) fn parse_identifier_reference<const YIELD: bool, const AWAIT: bool>(
    tokenizer: &mut Tokenizer<'_>,
) -> Result<String, SyntaxError> {
    if let Ok(identifier) = tokenizer.attempt(Identifier::parse) {
        return Ok(identifier.0);
    }

    if YIELD && tokenizer.expect_keyword("yield").is_ok() {
        return Ok("yield".to_string());
    }

    if AWAIT && tokenizer.expect_keyword("await").is_ok() {
        return Ok("await".to_string());
    }

    Err(tokenizer.syntax_error())
}
