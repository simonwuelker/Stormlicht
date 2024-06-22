//! <https://drafts.csswg.org/css-lists/#propdef-list-style-type>

use crate::{
    css::{
        style::{computed, StyleContext, ToComputedStyle},
        syntax::Token,
        values::CounterStyle,
        CSSParse, ParseError, Parser,
    },
    static_interned, InternedString,
};

/// <https://drafts.csswg.org/css-lists/#propdef-list-style-type>
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListStyleType {
    CounterStyle(CounterStyle),
    String(InternedString),
    None,
}

impl ListStyleType {
    #[must_use]
    pub fn as_str(&self) -> Option<String> {
        match self {
            Self::CounterStyle(counter) => Some(counter.as_str()),
            Self::String(s) => Some(s.to_string()),
            Self::None => None,
        }
    }
}

impl<'a> CSSParse<'a> for ListStyleType {
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        let list_style_type = match parser.next_token_ignoring_whitespace() {
            Some(Token::Ident(static_interned!("none"))) => Self::None,
            Some(Token::Ident(static_interned!("decimal"))) => {
                Self::CounterStyle(CounterStyle::Decimal)
            },
            Some(Token::Ident(static_interned!("disc"))) => Self::CounterStyle(CounterStyle::Disc),
            Some(Token::Ident(static_interned!("square"))) => {
                Self::CounterStyle(CounterStyle::Square)
            },
            Some(Token::Ident(static_interned!("disclosure-open"))) => {
                Self::CounterStyle(CounterStyle::DisclosureOpen)
            },
            Some(Token::Ident(static_interned!("disclosure-closed"))) => {
                Self::CounterStyle(CounterStyle::DisclosureClosed)
            },
            Some(Token::String(s)) => Self::String(s),
            _ => return Err(ParseError),
        };

        Ok(list_style_type)
    }
}

impl ToComputedStyle for ListStyleType {
    type Computed = computed::ListStyleType;

    fn to_computed_style(&self, context: &StyleContext) -> Self::Computed {
        _ = context;

        self.clone()
    }
}
