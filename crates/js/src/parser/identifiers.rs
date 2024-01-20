use super::{SyntaxError, Tokenizer};

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
        identifier.0
    } else {
        let identifier_name = tokenizer.consume_identifier()?;

        if matches!(identifier_name.as_str(), "yield" | "await") {
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

impl Identifier {
    /// <https://262.ecma-international.org/14.0/#prod-Identifier>
    pub(crate) fn parse(tokenizer: &mut Tokenizer<'_>) -> Result<Self, SyntaxError> {
        let identifier_name = tokenizer.consume_identifier()?;
        if RESERVED_WORDS.contains(&identifier_name.as_str()) {
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

    if YIELD && tokenizer.consume_keyword("yield").is_ok() {
        return Ok("yield".to_string());
    }

    if AWAIT && tokenizer.consume_keyword("await").is_ok() {
        return Ok("await".to_string());
    }

    Err(tokenizer.syntax_error())
}
