use crate::{
    css::{self, CSSParse},
    static_interned, InternedString,
};

use super::{AutoOr, Length, PercentageOr};

/// The value of an [inset property](https://drafts.csswg.org/css-position/#inset-properties)
pub type Inset = AutoOr<PercentageOr<Length>>;

/// <https://drafts.csswg.org/css-align-3/#typedef-overflow-position>
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverflowPosition {
    /// <https://drafts.csswg.org/css-align-3/#valdef-overflow-position-safe>
    Safe,

    /// <https://drafts.csswg.org/css-align-3/#valdef-overflow-position-unsafe>
    Unsafe,

    Unspecified,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JustifySelfPosition {
    Center,
    Start,
    End,
    SelfStart,
    SelfEnd,
    FlexStart,
    FlexEnd,
    Left,
    Right,
}

/// <https://drafts.csswg.org/css-align-3/#propdef-justify-self>
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JustifySelf {
    /// <https://drafts.csswg.org/css-align-3/#valdef-justify-self-normal>
    Normal,

    /// <https://drafts.csswg.org/css-align-3/#valdef-justify-self-stretch>
    Stretch,

    /// <https://drafts.csswg.org/css-align-3/#valdef-justify-self-first-baseline>
    FirstBaseline,

    /// <https://drafts.csswg.org/css-align-3/#valdef-justify-self-last-baseline>
    LastBaseline,
    // FIXME: There are more values here
    SelfPosition(OverflowPosition, JustifySelfPosition),
}

impl<'a> CSSParse<'a> for JustifySelf {
    fn parse(parser: &mut css::Parser<'a>) -> Result<Self, css::ParseError> {
        let justify_self = match parser.expect_identifier()? {
            static_interned!("normal") => Self::Normal,
            static_interned!("stretch") => Self::Stretch,
            static_interned!("first") => {
                let static_interned!("baseline") = parser.expect_identifier()? else {
                    return Err(css::ParseError);
                };
                Self::FirstBaseline
            },
            static_interned!("last") => {
                let static_interned!("baseline") = parser.expect_identifier()? else {
                    return Err(css::ParseError);
                };
                Self::LastBaseline
            },
            static_interned!("safe") => {
                let position = parser.parse()?;
                Self::SelfPosition(OverflowPosition::Safe, position)
            },
            static_interned!("unsafe") => {
                let position = parser.parse()?;
                Self::SelfPosition(OverflowPosition::Unsafe, position)
            },
            other => {
                let position = JustifySelfPosition::from_identifier(other)?;
                Self::SelfPosition(OverflowPosition::Unspecified, position)
            },
        };

        Ok(justify_self)
    }
}

impl JustifySelfPosition {
    fn from_identifier(ident: InternedString) -> Result<Self, css::ParseError> {
        let position = match ident {
            static_interned!("center") => Self::Center,
            static_interned!("start") => Self::Start,
            static_interned!("end") => Self::End,
            static_interned!("self-start") => Self::SelfStart,
            static_interned!("self-end") => Self::SelfEnd,
            static_interned!("flex-start") => Self::FlexStart,
            static_interned!("flex-end") => Self::FlexEnd,
            static_interned!("left") => Self::Left,
            static_interned!("right") => Self::Right,
            _ => return Err(css::ParseError),
        };

        Ok(position)
    }
}

impl<'a> CSSParse<'a> for JustifySelfPosition {
    fn parse(parser: &mut css::Parser<'a>) -> Result<Self, css::ParseError> {
        Self::from_identifier(parser.expect_identifier()?)
    }
}
