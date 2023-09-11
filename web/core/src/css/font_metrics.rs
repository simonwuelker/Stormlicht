use font::Font;

use crate::FONT_CACHE;

use super::layout::CSSPixels;

pub const DEFAULT_FONT_SIZE: CSSPixels = CSSPixels(16.0);

#[derive(Clone, Copy)]
pub struct FontMetrics<'a> {
    pub font_face: &'a Font,
    pub size: CSSPixels,
}

impl<'a> Default for FontMetrics<'a> {
    fn default() -> Self {
        Self {
            font_face: FONT_CACHE.fallback(),
            size: DEFAULT_FONT_SIZE,
        }
    }
}
