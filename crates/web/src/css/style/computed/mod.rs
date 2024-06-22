mod line_height;

use crate::css::{
    layout::Pixels,
    values::{AutoOr, PercentageOr},
};

use super::specified;

pub use line_height::LineHeight;

/// </// <https://drafts.csswg.org/css-backgrounds/#background-color>>
pub type BackgroundColor = specified::BackgroundColor;

/// <https://drafts.csswg.org/css-backgrounds/#background-image>
pub type BackgroundImage = specified::BackgroundImage;

/// <https://drafts.csswg.org/css2/#border-style-properties>
pub type BorderStyle = specified::LineStyle;

/// <https://drafts.csswg.org/css2/#border-width-properties>
pub type BorderWidth = Pixels;

/// <https://drafts.csswg.org/css2/#font-size-props>
pub type FontSize = Pixels;

/// <https://drafts.csswg.org/css-position/#inset-properties>
pub type Inset = AutoOr<PercentageOr<Length>>;

/// <https://drafts.csswg.org/css-align-3/#propdef-justify-self>
pub type JustifySelf = specified::JustifySelf;

pub type Length = Pixels;

/// <https://drafts.csswg.org/css2/#value-def-margin-width>
pub type Margin = AutoOr<PercentageOr<Length>>;

/// <https://drafts.csswg.org/css2/#value-def-padding-width>
pub type Padding = PercentageOr<Length>;

/// <https://drafts.csswg.org/css2/#propdef-vertical-align>
pub type VerticalAlign = specified::VerticalAlign;
