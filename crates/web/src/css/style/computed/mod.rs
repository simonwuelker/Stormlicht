use crate::css::{
    layout::Pixels,
    values::{AutoOr, PercentageOr},
};

use super::specified;

/// <https://drafts.csswg.org/css2/#border-style-properties>
pub type BorderStyle = specified::LineStyle;

/// <https://drafts.csswg.org/css2/#border-width-properties>
pub type BorderWidth = Pixels;

/// <https://drafts.csswg.org/css2/#font-size-props>
pub type FontSize = Pixels;

pub type Length = Pixels;

/// <https://drafts.csswg.org/css2/#propdef-line-height>
pub type LineHeight = Pixels;

/// <https://drafts.csswg.org/css2/#value-def-margin-width>
pub type Margin = AutoOr<PercentageOr<Length>>;

/// <https://drafts.csswg.org/css2/#value-def-padding-width>
pub type Padding = PercentageOr<Length>;
