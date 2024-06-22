use crate::{
    css::{
        style::{computed, StyleContext, ToComputedStyle},
        CSSParse, ParseError, Parser,
    },
    static_interned,
};

/// <https://drafts.csswg.org/css2/#propdef-float>
#[derive(Clone, Copy, Debug, Default)]
pub struct Float {
    side: Option<FloatSide>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FloatSide {
    Left,
    Right,
}

impl Float {
    #[must_use]
    pub fn side(&self) -> Option<FloatSide> {
        self.side
    }
}

impl<'a> CSSParse<'a> for Float {
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        let side = match parser.expect_identifier()? {
            static_interned!("left") => Some(FloatSide::Left),
            static_interned!("right") => Some(FloatSide::Right),
            static_interned!("none") => None,
            _ => return Err(ParseError),
        };
        Ok(Self { side })
    }
}

impl ToComputedStyle for Float {
    type Computed = computed::Float;

    fn to_computed_style(&self, context: &StyleContext) -> Self::Computed {
        _ = context;

        *self
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

impl ToComputedStyle for Clear {
    type Computed = computed::Clear;

    fn to_computed_style(&self, context: &StyleContext) -> Self::Computed {
        _ = context;

        *self
    }
}
