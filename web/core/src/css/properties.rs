use super::{syntax::Token, values::color::Color, CSSParse, ParseError, Parser};
use cssproperty_derive::CSSProperty;

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
    pub is_important: bool,
}

impl StyleProperty {
    pub fn parse_value(parser: &mut Parser, property_name: &str) -> Result<Self, ParseError> {
        let property = match property_name {
            "color" => Self::Color(ColorValue::parse(parser)?),
            "background-color" => Self::BackgroundColor(BackgroundColorValue::parse(parser)?),
            "display" => Self::Display(DisplayValue::parse(parser)?),
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

/// <https://drafts.csswg.org/css-display/#the-display-properties>
#[derive(Clone, Debug, CSSProperty)]
pub enum DisplayValue {
    // FIXME: missing values
    #[unordered]
    InsideOutside {
        outside: DisplayOutside,
        inside: DisplayInside,
    },
}

/// <https://drafts.csswg.org/css-display/#typedef-display-outside>
#[derive(Clone, Copy, Debug, PartialEq, Eq, CSSProperty)]
pub enum DisplayOutside {
    #[keyword = "block"]
    Block,

    #[keyword = "inline"]
    Inline,

    #[keyword = "run-in"]
    RunIn,
}

/// <https://drafts.csswg.org/css-display/#typedef-display-inside>
#[derive(Clone, Copy, Debug, PartialEq, Eq, CSSProperty)]
pub enum DisplayInside {
    #[keyword = "flow"]
    Flow,

    #[keyword = "flow-root"]
    FlowRoot,

    #[keyword = "table"]
    Table,

    #[keyword = "flex"]
    Flex,

    #[keyword = "grid"]
    Grid,

    #[keyword = "ruby"]
    Ruby,
}
