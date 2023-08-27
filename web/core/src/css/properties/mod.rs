mod background_color;
mod color;
mod display;

pub use background_color::BackgroundColorValue;
pub use color::ColorValue;
pub use display::{
    DisplayBox, DisplayInside, DisplayInsideOutside, DisplayInternal, DisplayOutside, DisplayValue,
};

use super::{
    values::{AutoOr, LengthPercentage},
    CSSParse, ParseError, Parser,
};

use string_interner::{static_interned, static_str, InternedString};

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
    Color(ColorValue),

    /// <https://drafts.csswg.org/css2/#background-properties>
    BackgroundColor(BackgroundColorValue),

    /// <https://drafts.csswg.org/css-display/#the-display-properties>
    Display(DisplayValue),

    /// <https://drafts.csswg.org/css-box-3/#propdef-margin-top>
    MarginTop(AutoOr<LengthPercentage>),

    /// <https://drafts.csswg.org/css-box-3/#propdef-margin-right>
    MarginRight(AutoOr<LengthPercentage>),

    /// <https://drafts.csswg.org/css-box-3/#propdef-margin-bottom>
    MarginBottom(AutoOr<LengthPercentage>),

    /// <https://drafts.csswg.org/css-box-3/#propdef-margin-left>
    MarginLeft(AutoOr<LengthPercentage>),

    /// <https://drafts.csswg.org/css2/#propdef-width>
    Width(AutoOr<LengthPercentage>),

    /// <https://drafts.csswg.org/css2/#propdef-height>
    Height(AutoOr<LengthPercentage>),
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
        parser: &mut Parser,
        property_name: InternedString,
    ) -> Result<Self, ParseError> {
        let property = match property_name {
            static_interned!("color") => Self::Color(ColorValue::parse(parser)?),
            static_interned!("background-color") => {
                Self::BackgroundColor(BackgroundColorValue::parse(parser)?)
            },
            static_interned!("display") => Self::Display(CSSParse::parse(parser)?),
            static_interned!("margin-top") => Self::MarginTop(CSSParse::parse(parser)?),
            static_interned!("margin-right") => Self::MarginRight(CSSParse::parse(parser)?),
            static_interned!("margin-bottom") => Self::MarginBottom(CSSParse::parse(parser)?),
            static_interned!("margin-left") => Self::MarginLeft(CSSParse::parse(parser)?),
            static_interned!("width") => Self::Width(CSSParse::parse(parser)?),
            static_interned!("height") => Self::Width(CSSParse::parse(parser)?),
            _ => {
                log::warn!("Unknown CSS property name: {:?}", property_name.to_string());
                return Err(ParseError);
            },
        };
        Ok(property)
    }
}
