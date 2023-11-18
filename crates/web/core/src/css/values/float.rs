use crate::{
    css::{CSSParse, ParseError, Parser},
    static_interned,
};

/// <https://drafts.csswg.org/css2/#propdef-float>
#[derive(Clone, Copy, Debug)]
pub enum Float {
    Left,
    Right,
    None,
}

impl<'a> CSSParse<'a> for Float {
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        let value = match parser.expect_identifier()? {
            static_interned!("left") => Self::Left,
            static_interned!("right") => Self::Right,
            static_interned!("none") => Self::None,
            _ => return Err(ParseError),
        };
        Ok(value)
    }
}

/// <https://drafts.csswg.org/css2/#propdef-clear>
#[derive(Clone, Copy, Debug)]
pub enum Clear {
    None,
    Left,
    Right,
    Both,
}

impl<'a> CSSParse<'a> for Clear {
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        let value = match parser.expect_identifier()? {
            static_interned!("none") => Self::None,
            static_interned!("left") => Self::Left,
            static_interned!("right") => Self::Right,
            static_interned!("both") => Self::Both,
            _ => return Err(ParseError),
        };
        Ok(value)
    }
}
