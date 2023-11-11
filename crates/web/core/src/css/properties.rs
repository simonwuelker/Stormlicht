use crate::{
    css::{
        values::{
            AutoOr, BackgroundColor, Color, Display, FontFamily, FontSize, Length, PercentageOr,
            Position,
        },
        CSSParse, ParseError, Parser,
    },
    static_interned, InternedString,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Important {
    Yes,
    #[default]
    No,
}

/// Enumerates the CSS properties supported by the user agent
#[derive(Clone, Debug)]
pub enum StyleProperty {
    /// <https://drafts.csswg.org/css2/#colors>
    Color(Color),

    /// <https://drafts.csswg.org/css2/#background-properties>
    BackgroundColor(BackgroundColor),

    /// <https://drafts.csswg.org/css-display/#the-display-properties>
    Display(Display),

    /// <https://drafts.csswg.org/css-fonts/#font-family-prop>
    FontFamily(FontFamily),

    /// <https://drafts.csswg.org/css2/#font-size-props>
    FontSize(FontSize),

    /// <https://drafts.csswg.org/css-box-3/#propdef-margin-top>
    MarginTop(AutoOr<PercentageOr<Length>>),

    /// <https://drafts.csswg.org/css-box-3/#propdef-margin-right>
    MarginRight(AutoOr<PercentageOr<Length>>),

    /// <https://drafts.csswg.org/css-box-3/#propdef-margin-bottom>
    MarginBottom(AutoOr<PercentageOr<Length>>),

    /// <https://drafts.csswg.org/css-box-3/#propdef-margin-left>
    MarginLeft(AutoOr<PercentageOr<Length>>),

    /// <https://drafts.csswg.org/css2/#propdef-width>
    Width(AutoOr<PercentageOr<Length>>),

    /// <https://drafts.csswg.org/css2/#propdef-height>
    Height(AutoOr<PercentageOr<Length>>),

    /// <https://drafts.csswg.org/css2/#propdef-padding-top>
    PaddingTop(PercentageOr<Length>),

    /// <https://drafts.csswg.org/css2/#propdef-padding-right>
    PaddingRight(PercentageOr<Length>),

    /// <https://drafts.csswg.org/css2/#propdef-padding-bottom>
    PaddingBottom(PercentageOr<Length>),

    /// <https://drafts.csswg.org/css2/#propdef-padding-left>
    PaddingLeft(PercentageOr<Length>),

    /// <https://drafts.csswg.org/css-position/#position-property>
    Position(Position),
}

#[derive(Clone, Debug)]
pub struct StylePropertyDeclaration {
    pub value: StyleProperty,

    /// Whether or not the property was declared with `!important`.
    ///
    /// For example: `color: red!important;`
    pub important: Important,
}

impl StyleProperty {
    pub fn parse_value(
        parser: &mut Parser<'_>,
        property_name: InternedString,
    ) -> Result<Self, ParseError> {
        let property = match property_name {
            static_interned!("color") => Self::Color(CSSParse::parse(parser)?),
            static_interned!("background-color") => {
                Self::BackgroundColor(BackgroundColor::parse(parser)?)
            },
            static_interned!("display") => Self::Display(CSSParse::parse(parser)?),
            static_interned!("font-family") => Self::FontFamily(CSSParse::parse(parser)?),
            static_interned!("font-size") => Self::FontSize(CSSParse::parse(parser)?),
            static_interned!("margin-top") => Self::MarginTop(CSSParse::parse(parser)?),
            static_interned!("margin-right") => Self::MarginRight(CSSParse::parse(parser)?),
            static_interned!("margin-bottom") => Self::MarginBottom(CSSParse::parse(parser)?),
            static_interned!("margin-left") => Self::MarginLeft(CSSParse::parse(parser)?),
            static_interned!("padding-top") => Self::PaddingTop(CSSParse::parse(parser)?),
            static_interned!("padding-right") => Self::PaddingRight(CSSParse::parse(parser)?),
            static_interned!("padding-bottom") => Self::PaddingBottom(CSSParse::parse(parser)?),
            static_interned!("padding-left") => Self::PaddingLeft(CSSParse::parse(parser)?),
            static_interned!("width") => Self::Width(CSSParse::parse(parser)?),
            static_interned!("height") => Self::Height(CSSParse::parse(parser)?),
            static_interned!("position") => Self::Position(CSSParse::parse(parser)?),
            _ => {
                log::warn!("Unknown CSS property name: {:?}", property_name.to_string());
                return Err(ParseError);
            },
        };
        Ok(property)
    }
}
