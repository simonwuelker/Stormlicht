mod background_color;
mod color;
mod display;

pub use background_color::BackgroundColorValue;
pub use color::ColorValue;
pub use display::DisplayValue;

use super::{CSSParse, ParseError, Parser};

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
            static_interned!("display") => Self::Display(DisplayValue::parse(parser)?),
            _ => {
                log::warn!("Unknown CSS property name: {:?}", property_name.to_string());
                return Err(ParseError);
            },
        };
        Ok(property)
    }
}
