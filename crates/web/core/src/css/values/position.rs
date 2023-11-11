use crate::{
    css::{syntax::Token, CSSParse, ParseError, Parser},
    static_interned,
};

/// <https://drafts.csswg.org/css-position/#position-property>
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Position {
    /// <https://drafts.csswg.org/css-position/#valdef-position-static>
    #[default]
    Static,

    /// <https://drafts.csswg.org/css-position/#valdef-position-relative>
    Relative,

    /// <https://drafts.csswg.org/css-position/#valdef-position-sticky>
    Sticky,

    /// <https://drafts.csswg.org/css-position/#valdef-position-absolute>
    Absolute,

    /// <https://drafts.csswg.org/css-position/#valdef-position-fixed>
    Fixed,
}

impl Position {
    pub fn is_absolute(&self) -> bool {
        *self == Self::Absolute
    }
}

impl<'a> CSSParse<'a> for Position {
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        let position = match parser.next_token() {
            Some(Token::Ident(static_interned!("static"))) => Self::Static,
            Some(Token::Ident(static_interned!("relative"))) => Self::Relative,
            Some(Token::Ident(static_interned!("sticky"))) => Self::Sticky,
            Some(Token::Ident(static_interned!("absolute"))) => Self::Absolute,
            Some(Token::Ident(static_interned!("fixed"))) => Self::Fixed,
            _ => return Err(ParseError),
        };

        Ok(position)
    }
}
