mod border;
mod line_height;

use crate::css::{
    layout::Pixels,
    values::{AutoOr, PercentageOr},
};

use super::specified;

pub use border::Border;
pub use line_height::LineHeight;

/// </// <https://drafts.csswg.org/css-backgrounds/#background-color>>
pub type BackgroundColor = specified::BackgroundColor;

/// <https://drafts.csswg.org/css-backgrounds/#background-image>
pub type BackgroundImage = specified::BackgroundImage;

/// <https://drafts.csswg.org/css2/#border-style-properties>
pub type BorderStyle = specified::LineStyle;

/// <https://drafts.csswg.org/css2/#border-width-properties>
pub type BorderWidth = Pixels;

/// <https://drafts.csswg.org/css2/#propdef-clear>
pub type Clear = specified::Clear;

/// <https://drafts.csswg.org/css-ui/#cursor>
pub type Cursor = specified::Cursor;

/// <https://drafts.csswg.org/css-display/#the-display-properties>
pub type Display = specified::Display;

/// <https://drafts.csswg.org/css2/#propdef-float>
pub type Float = specified::Float;

/// <https://drafts.csswg.org/css-fonts/#font-family-prop>
pub type FontFamily = specified::FontFamily;

/// <https://drafts.csswg.org/css2/#font-size-props>
pub type FontSize = Pixels;

/// <https://drafts.csswg.org/css-fonts/#font-style-prop>
pub type FontStyle = specified::FontStyle;

/// <https://drafts.csswg.org/css-position/#inset-properties>
pub type Inset = AutoOr<PercentageOr<Length>>;

/// <https://drafts.csswg.org/css-align-3/#propdef-justify-self>
pub type JustifySelf = specified::JustifySelf;

pub type Length = Pixels;

/// <https://drafts.csswg.org/css-backgrounds/#typedef-line-style>
pub type LineStyle = specified::LineStyle;

/// <https://drafts.csswg.org/css-backgrounds/#typedef-line-width>
pub type LineWidth = Pixels;

/// <https://drafts.csswg.org/css-lists/#propdef-list-style-type>
pub type ListStyleType = specified::ListStyleType;

/// <https://drafts.csswg.org/css2/#value-def-margin-width>
pub type Margin = AutoOr<PercentageOr<Length>>;

/// <https://drafts.csswg.org/css2/#value-def-padding-width>
pub type Padding = PercentageOr<Length>;

/// <https://drafts.csswg.org/css-position/#position-property>
pub type Position = specified::Position;

/// <https://drafts.csswg.org/css2/#propdef-vertical-align>
pub type VerticalAlign = specified::VerticalAlign;
