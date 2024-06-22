//! <https://drafts.csswg.org/css2/#propdef-line-height>

use crate::css::{layout::Pixels, values::Number};

/// The default font-size multiplier for `line-height: normal`.
///
/// The spec recommends using a value between `1.0` and `1.2` here.
/// (<https://drafts.csswg.org/css2/#valdef-line-height-normal>)
const LINE_HEIGHT_NORMAL: f32 = 1.0;

/// <https://drafts.csswg.org/css2/#propdef-line-height>
#[derive(Clone, Copy, Debug)]
pub enum LineHeight {
    Absolute(Pixels),
    Relative(Number),
    Normal,
}

impl LineHeight {
    #[must_use]
    pub fn used_value(&self, font_size: super::FontSize) -> Pixels {
        match self {
            Self::Absolute(absolute) => *absolute,
            Self::Relative(relative) => f32::from(*relative) * font_size,
            Self::Normal => LINE_HEIGHT_NORMAL * font_size,
        }
    }
}
