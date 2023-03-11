//! Defines the CSS tree produced by the [Parser](crate::parser::Parser)
//!
//! See <https://drafts.csswg.org/css-syntax/#parsing> for more details.

use std::borrow::Cow;

use crate::tokenizer::{HashFlag, Number};

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
pub struct SimpleBlock<'a> {
    delimiter: BlockDelimiter,
    value: Vec<ComponentValue<'a>>,
}

/// https://drafts.csswg.org/css-syntax/#function
#[derive(Clone, Debug)]
pub struct Function<'a> {
    name: Cow<'a, str>,
    body: Vec<ComponentValue<'a>>,
}

#[derive(Clone, Debug)]
pub enum ComponentValue<'a> {
    Block(SimpleBlock<'a>),
    Function(Function<'a>),
    Token(PreservedToken<'a>),
}

/// https://drafts.csswg.org/css-syntax/#preserved-tokens
#[derive(Clone, Debug)]
pub enum PreservedToken<'a> {
    Ident(Cow<'a, str>),
    AtKeyword(Cow<'a, str>),
    String(Cow<'a, str>),
    BadString(Cow<'a, str>),
    BadURI(Cow<'a, str>),
    Hash(Cow<'a, str>, HashFlag),
    Number(Number),
    Percentage(Number),
    Dimension(Number, Cow<'a, str>),
    URI(Cow<'a, str>),
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
}

/// https://drafts.csswg.org/css-syntax/#declaration
#[derive(Clone, Debug)]
pub struct Declaration<'a> {
    name: Cow<'a, str>,
    value: Vec<ComponentValue<'a>>,
    is_important: bool,
}

/// https://drafts.csswg.org/css-syntax/#at-rule
#[derive(Clone, Debug)]
pub struct AtRule<'a> {
    name: Cow<'a, str>,
    prelude: Vec<ComponentValue<'a>>,
    block: Option<SimpleBlock<'a>>,
}

/// https://drafts.csswg.org/css-syntax/#qualified-rule
#[derive(Clone, Debug)]
pub struct QualifiedRule<'a> {
    prelude: Vec<ComponentValue<'a>>,
    block: SimpleBlock<'a>,
}

#[derive(Clone, Debug)]
pub enum Rule<'a> {
    QualifiedRule(QualifiedRule<'a>),
    AtRule(AtRule<'a>),
}

impl<'a> SimpleBlock<'a> {
    pub fn new(delimiter: BlockDelimiter, value: Vec<ComponentValue<'a>>) -> Self {
        Self { delimiter, value }
    }

    pub fn delimiter(&self) -> BlockDelimiter {
        self.delimiter
    }

    pub fn values(&self) -> &[ComponentValue<'a>] {
        &self.value
    }
}

impl<'a> Function<'a> {
    pub fn new(name: Cow<'a, str>, body: Vec<ComponentValue<'a>>) -> Self {
        Self { name, body }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn body(&self) -> &[ComponentValue<'a>] {
        &self.body
    }
}

impl<'a> Declaration<'a> {
    pub fn new(name: Cow<'a, str>, value: Vec<ComponentValue<'a>>, is_important: bool) -> Self {
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

    pub fn value(&self) -> &[ComponentValue<'a>] {
        &self.value
    }
}

impl<'a> AtRule<'a> {
    pub fn new(
        name: Cow<'a, str>,
        prelude: Vec<ComponentValue<'a>>,
        block: Option<SimpleBlock<'a>>,
    ) -> Self {
        Self {
            name,
            prelude,
            block,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn prelude(&self) -> &[ComponentValue<'a>] {
        &self.prelude
    }

    pub fn block(&self) -> Option<&SimpleBlock> {
        self.block.as_ref()
    }
}

impl<'a> QualifiedRule<'a> {
    pub fn new(prelude: Vec<ComponentValue<'a>>, block: SimpleBlock<'a>) -> Self {
        Self { prelude, block }
    }

    pub fn prelude(&self) -> &[ComponentValue<'a>] {
        &self.prelude
    }

    pub fn block(&self) -> &SimpleBlock {
        &self.block
    }
}

impl<'a> Rule<'a> {
    pub fn prelude(&self) -> &[ComponentValue<'a>] {
        match self {
            Self::AtRule(at_rule) => at_rule.prelude(),
            Self::QualifiedRule(qualified_rule) => qualified_rule.prelude(),
        }
    }
}
