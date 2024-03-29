/// <https://262.ecma-international.org/14.0/#sec-punctuators>
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Punctuator {
    /// <https://262.ecma-international.org/14.0/#prod-OptionalChainingPunctuator>
    OptionalChaining,

    CurlyBraceOpen,
    ParenthesisOpen,
    ParenthesisClose,
    BracketOpen,
    BracketClose,
    Dot,
    TripleDot,
    Semicolon,
    Comma,
    LessThan,
    GreaterThan,
    LessThanEqual,
    GreaterThanEqual,
    DoubleEqual,
    ExclamationMarkEqual,
    TripleEqual,
    ExclamationMarkDoubleEqual,
    Plus,
    Minus,
    Asterisk,
    Percent,
    DoubleAsterisk,
    DoublePlus,
    DoubleMinus,
    DoubleLessThan,
    DoubleGreaterThan,
    TripleGreaterThan,
    Ampersand,
    VerticalBar,
    Caret,
    ExclamationMark,
    Tilde,
    DoubleAmpersand,
    DoubleVerticalBar,
    DoubleQuestionMark,
    QuestionMark,
    Colon,
    Equal,
    PlusEqual,
    MinusEqual,
    AsteriskEqual,
    PercentEqual,
    DoubleAsteriskEqual,
    DoubleLessThanEqual,
    DoubleGreaterThanEqual,
    TripleGreaterThanEqual,
    AmpersandEqual,
    VerticalBarEqual,
    CaretEqual,
    DoubleAmpersandEqual,
    DoubleVerticalBarEqual,
    DoubleQuestionMarkEqual,
    EqualGreaterThan,

    Slash,
    SlashEqual,

    CurlyBraceClose,
}

/// <https://262.ecma-international.org/14.0/#sec-tokens>
#[derive(Clone, Debug)]
pub enum Token {
    /// <https://262.ecma-international.org/14.0/#prod-IdentifierName>
    Identifier(String),

    /// <https://262.ecma-international.org/14.0/#prod-PrivateIdentifier>
    PrivateIdentifier(String),

    /// <https://262.ecma-international.org/14.0/#prod-Punctuator>
    Punctuator(Punctuator),

    /// <https://262.ecma-international.org/14.0/#prod-NullLiteral>
    NullLiteral,

    /// <https://262.ecma-international.org/14.0/#prod-BooleanLiteral>
    BooleanLiteral(bool),

    /// <https://262.ecma-international.org/14.0/#prod-NumericLiteral>
    NumericLiteral(u32),

    /// <https://262.ecma-international.org/14.0/#prod-StringLiteral>
    StringLiteral(String),

    /// <https://262.ecma-international.org/14.0/#prod-Template>
    Template,

    LineTerminator,
}

impl Token {
    #[must_use]
    pub const fn is_line_terminator(&self) -> bool {
        matches!(self, Self::LineTerminator)
    }

    #[must_use]
    pub fn is_identifier(&self, want: &str) -> bool {
        match self {
            Self::Identifier(ident) if ident == want => true,
            _ => false,
        }
    }

    #[must_use]
    pub fn is_punctuator(&self, want: Punctuator) -> bool {
        match self {
            Self::Punctuator(p) if *p == want => true,
            _ => false,
        }
    }
}
