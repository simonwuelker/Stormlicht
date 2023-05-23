use super::values::color::Color;

/// Enumerates the CSS properties supported by the user agent
#[derive(Clone, Debug)]
pub enum StyleProperty {
    /// <https://drafts.csswg.org/css2/#colors>
    Color(ColorValue),

    /// <https://drafts.csswg.org/css2/#background-properties>
    BackgroundColor(BackgroundColorValue),
}

/// <https://drafts.csswg.org/css2/#colors>
#[derive(Clone, Debug)]
pub enum ColorValue {
    Color(Color),
    Inherit,
}

/// <https://drafts.csswg.org/css2/#background-properties>
#[derive(Clone, Debug)]
pub enum BackgroundColorValue {
    Color(Color),
    Transparent,
    Inherit,
}
