//! <https://drafts.csswg.org/css-backgrounds/#propdef-border>

use super::{LineStyle, LineWidth};
use crate::css::values::Color;

/// <https://drafts.csswg.org/css-backgrounds/#propdef-border>
pub struct Border {
    pub color: Color,
    pub width: LineWidth,
    pub style: LineStyle,
}
