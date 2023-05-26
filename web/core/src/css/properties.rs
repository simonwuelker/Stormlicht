use super::{syntax::Token, values::color::Color, CSSParse, ParseError, Parser};
use cssproperty_derive::CSSProperty;

/// Enumerates the CSS properties supported by the user agent
#[derive(Clone, Debug)]
pub enum StyleProperty {
    /// <https://drafts.csswg.org/css2/#colors>
    Color(ColorValue),

    /// <https://drafts.csswg.org/css2/#background-properties>
    BackgroundColor(BackgroundColorValue),
}

impl StyleProperty {
    pub fn parse_value(parser: &mut Parser, property_name: &str) -> Result<Self, ParseError> {
        let property = match property_name {
            "color" => Self::Color(ColorValue::parse(parser)?),
            "background-color" => Self::BackgroundColor(BackgroundColorValue::parse(parser)?),
            _ => {
                log::warn!("Unknown CSS property name: {property_name:?}");
                return Err(ParseError);
            },
        };
        Ok(property)
    }
}

/// <https://drafts.csswg.org/css2/#colors>
#[derive(Clone, Debug, CSSProperty)]
pub enum ColorValue {
    Color(Color),

    #[keyword = "inherit"]
    Inherit,
}

/// <https://drafts.csswg.org/css2/#background-properties>
#[derive(Clone, Debug, CSSProperty)]
pub enum BackgroundColorValue {
    Color(Color),

    #[keyword = "transparent"]
    Transparent,

    #[keyword = "inherit"]
    Inherit,
}
