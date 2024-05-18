//! <https://262.ecma-international.org/14.0/#sec-identifiers>

use super::{
    tokenization::{GoalSymbol, SkipLineTerminators, Token, Tokenizer},
    SyntaxError,
};

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
    let Some(Token::Identifier(identifier)) = tokenizer.next(SkipLineTerminators::Yes)? else {
        return Err(tokenizer.syntax_error("expected identifier"));
    };

    if !YIELD && identifier.as_str() == "yield" {
        return Err(tokenizer.syntax_error("\"yield\" is not a valid identifier here"));
    }

    if !AWAIT && identifier.as_str() == "await" {
        return Err(tokenizer.syntax_error("\"await\" is not a valid identifier here"));
    }

    if tokenizer.is_strict() && matches!(identifier.as_str(), "arguments" | "eval") {
        return Err(tokenizer.syntax_error(format!(
            "{:?} is not a valid identifier here",
            identifier.as_str()
        )));
    }

    Ok(identifier)
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
        let Some(Token::Identifier(identifier_name)) = tokenizer.next(SkipLineTerminators::Yes)?
        else {
            return Err(tokenizer.syntax_error("expected identifier"));
        };

        if RESERVED_WORDS.contains(&identifier_name.as_str()) {
            return Err(
                tokenizer.syntax_error(format!("{:?} is a reserved identifier", identifier_name))
            );
        }

        if tokenizer.is_strict()
            && DISALLOWED_IDENTIFIERS_IN_STRICT_MODE.contains(&identifier_name.as_str())
        {
            return Err(tokenizer.syntax_error(format!(
                "{:?} is a not allowed as an identifier in strict mode",
                identifier_name
            )));
        }

        if tokenizer.goal_symbol() == GoalSymbol::Module && identifier_name.as_str() == "await" {
            return Err(tokenizer
                .syntax_error("\"await\" is a disallowed identifier when parsing a module"));
        }

        Ok(Self(identifier_name))
    }
}

/// <https://262.ecma-international.org/14.0/#prod-IdentifierReference>
pub(crate) fn parse_identifier_reference<const YIELD: bool, const AWAIT: bool>(
    tokenizer: &mut Tokenizer<'_>,
) -> Result<String, SyntaxError> {
    let next_token = tokenizer.peek(0, SkipLineTerminators::Yes)?;

    if let Some(Token::Identifier(ident)) = next_token {
        if YIELD && ident == "yield" {
            tokenizer.advance(1);
            return Ok("yield".to_string());
        }

        if AWAIT && ident == "await" {
            tokenizer.advance(1);
            return Ok("await".to_string());
        }

        Identifier::parse(tokenizer).map(|i| i.0)
    } else {
        Err(tokenizer.syntax_error("expected identifier"))
    }
}
