use crate::{
    css::{syntax::Token, CSSParse, ParseError, Parser},
    static_interned,
};

use std::fmt;

#[derive(Clone, Copy, PartialEq)]
pub struct Angle {
    /// Number of (full) turns described by this angle.
    ///
    /// May be negative.
    turns: f32,
}

impl Angle {
    #[must_use]
    pub const fn from_degrees(deg: f32) -> Self {
        Self { turns: deg / 360. }
    }

    #[must_use]
    pub const fn from_grad(grad: f32) -> Self {
        Self { turns: grad / 400. }
    }

    #[must_use]
    pub const fn from_rad(rad: f32) -> Self {
        Self {
            turns: rad / std::f32::consts::TAU,
        }
    }

    #[must_use]
    pub const fn from_turns(turns: f32) -> Self {
        Self { turns }
    }

    #[must_use]
    pub const fn as_degrees(&self) -> f32 {
        self.turns * 360.
    }
}

impl<'a> CSSParse<'a> for Angle {
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        let Some(Token::Dimension(value, dimension)) = parser.next_token_ignoring_whitespace()
        else {
            return Err(ParseError);
        };
        let value = f32::from(value);

        let angle = match dimension {
            static_interned!("deg") => {
                // https://drafts.csswg.org/css-values-4/#deg
                Self::from_degrees(value)
            },
            static_interned!("grad") => {
                // https://drafts.csswg.org/css-values-4/#grad
                Self::from_grad(value)
            },
            static_interned!("rad") => {
                // https://drafts.csswg.org/css-values-4/#rad
                Self::from_rad(value)
            },
            static_interned!("turn") => {
                // https://drafts.csswg.org/css-values-4/#turn
                Self::from_turns(value)
            },
            _ => {
                // Unknown angle unit
                return Err(ParseError);
            },
        };

        Ok(angle)
    }
}

impl fmt::Debug for Angle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}°", self.as_degrees())
    }
}
