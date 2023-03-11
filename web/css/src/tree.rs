//! Defines the CSS tree produced by the [Parser](crate::parser::Parser)
//!
//! See <https://drafts.csswg.org/css-syntax/#parsing> for more details.

use crate::tokenizer::{HashFlag, Number, Token};

#[derive(Clone, Copy, Debug)]
pub enum BlockDelimiter {
    /// `[ ... ]`
    Bracket,

    /// `{ ... }`
    CurlyBrace,

    /// `( ... )`
    Parenthesis,
}

/// https://drafts.csswg.org/css-syntax/#simple-block
#[derive(Clone, Debug)]
pub struct SimpleBlock {
    delimiter: BlockDelimiter,
    value: Vec<ComponentValue>,
}

/// https://drafts.csswg.org/css-syntax/#function
#[derive(Clone, Debug)]
pub struct Function {
    name: String,
    body: Vec<ComponentValue>,
}

#[derive(Clone, Debug)]
pub enum ComponentValue {
    Block(SimpleBlock),
    Function(Function),
    Token(PreservedToken),
}

/// https://drafts.csswg.org/css-syntax/#preserved-tokens
#[derive(Clone, Debug, PartialEq)]
pub enum PreservedToken {
    Ident(String),
    AtKeyword(String),
    String(String),
    BadString(String),
    BadURI(String),
    Hash(String, HashFlag),
    Number(Number),
    Percentage(Number),
    Dimension(Number, String),
    URI(String),
    CommentDeclarationOpen,
    CommentDeclarationClose,
    Colon,
    Semicolon,
    CurlyBraceClose,
    ParenthesisClose,
    BracketClose,
    Whitespace,
    Comma,
    Delim(char),
    EOF,
}

/// https://drafts.csswg.org/css-syntax/#declaration
#[derive(Clone, Debug)]
pub struct Declaration {
    name: String,
    value: Vec<ComponentValue>,
    is_important: bool,
}

/// https://drafts.csswg.org/css-syntax/#at-rule
#[derive(Clone, Debug)]
pub struct AtRule {
    name: String,
    prelude: Vec<ComponentValue>,
    block: Option<SimpleBlock>,
}

/// https://drafts.csswg.org/css-syntax/#qualified-rule
#[derive(Clone, Debug)]
pub struct QualifiedRule {
    prelude: Vec<ComponentValue>,
    block: SimpleBlock,
}

#[derive(Clone, Debug)]
pub enum Rule {
    QualifiedRule(QualifiedRule),
    AtRule(AtRule),
}

impl SimpleBlock {
    pub fn new(delimiter: BlockDelimiter, value: Vec<ComponentValue>) -> Self {
        Self { delimiter, value }
    }

    pub fn delimiter(&self) -> BlockDelimiter {
        self.delimiter
    }

    pub fn values(&self) -> &[ComponentValue] {
        &self.value
    }
}

impl Function {
    pub fn new(name: String, body: Vec<ComponentValue>) -> Self {
        Self { name, body }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn body(&self) -> &[ComponentValue] {
        &self.body
    }
}

impl Declaration {
    pub fn new(name: String, value: Vec<ComponentValue>, is_important: bool) -> Self {
        Self {
            name,
            value,
            is_important,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn is_important(&self) -> bool {
        self.is_important
    }

    pub fn value(&self) -> &[ComponentValue] {
        &self.value
    }
}

impl AtRule {
    pub fn new(name: String, prelude: Vec<ComponentValue>, block: Option<SimpleBlock>) -> Self {
        Self {
            name,
            prelude,
            block,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn prelude(&self) -> &[ComponentValue] {
        &self.prelude
    }

    pub fn block(&self) -> Option<&SimpleBlock> {
        self.block.as_ref()
    }
}

impl QualifiedRule {
    pub fn new(prelude: Vec<ComponentValue>, block: SimpleBlock) -> Self {
        Self { prelude, block }
    }

    pub fn prelude(&self) -> &[ComponentValue] {
        &self.prelude
    }

    pub fn block(&self) -> &SimpleBlock {
        &self.block
    }
}

impl Rule {
    pub fn prelude(&self) -> &[ComponentValue] {
        match self {
            Self::AtRule(at_rule) => at_rule.prelude(),
            Self::QualifiedRule(qualified_rule) => qualified_rule.prelude(),
        }
    }
}

impl BlockDelimiter {
    pub fn end_token(&self) -> Token<'static> {
        match self {
            BlockDelimiter::CurlyBrace => Token::CurlyBraceClose,
            BlockDelimiter::Parenthesis => Token::ParenthesisClose,
            BlockDelimiter::Bracket => Token::BracketClose,
        }
    }
}

impl PreservedToken {
    /// Converts from a regular [Token] to a [PreservedToken]. [PreservedTokens](PreservedToken)
    /// are a limited subset of [Tokens](Token).
    ///
    /// # Panic
    /// This function panics if the provided argument is not a valid [PreservedToken].
    /// This is the case for [Token::CurlyBraceOpen], [Token::BracketOpen], [Token::ParenthesisOpen],
    /// and [Token::Function].
    #[inline]
    pub fn from_regular_token(regular_token: Token<'_>) -> Self {
        match regular_token {
            Token::Ident(name) => Self::Ident(name.into_owned()),
            Token::AtKeyword(keyword) => Self::AtKeyword(keyword.into_owned()),
            Token::String(string) => Self::String(string.into_owned()),
            Token::BadString(bad_string) => Self::BadString(bad_string.into_owned()),
            Token::BadURI(bad_uri) => Self::BadURI(bad_uri.into_owned()),
            Token::Hash(hash, flag) => Self::Hash(hash.into_owned(), flag),
            Token::Number(number) => Self::Number(number),
            Token::Percentage(number) => Self::Percentage(number),
            Token::Dimension(number, unit) => Self::Dimension(number, unit.into_owned()),
            Token::URI(uri) => Self::URI(uri.into_owned()),
            Token::CommentDeclarationOpen => Self::CommentDeclarationOpen,
            Token::CommentDeclarationClose => Self::CommentDeclarationClose,
            Token::Colon => Self::Colon,
            Token::Semicolon => Self::Semicolon,
            Token::CurlyBraceClose => Self::CurlyBraceClose,
            Token::ParenthesisClose => Self::ParenthesisClose,
            Token::BracketClose => Self::BracketClose,
            Token::Whitespace => Self::Whitespace,
            Token::Comma => Self::Comma,
            Token::Delim(char) => Self::Delim(char),
            Token::EOF => Self::EOF,
            _ => panic!(
                "Trying to convert from {:?} to preserved token",
                regular_token
            ),
        }
    }
}
