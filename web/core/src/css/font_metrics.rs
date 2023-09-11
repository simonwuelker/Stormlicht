use font::Font;

use crate::FONT_CACHE;

use super::layout::CSSPixels;

pub const DEFAULT_FONT_SIZE: CSSPixels = CSSPixels(16.0);

#[derive(Clone, Debug)]
pub struct FontMetrics {
    pub font_face: Box<Font>,
    pub size: CSSPixels,
}

impl FontMetrics {
    #[inline]
    #[must_use]
    pub fn width_of(&self, text: &str) -> CSSPixels {
        CSSPixels(
            self.font_face
                .compute_rendered_width(text, self.size.into()),
        )
    }
}

impl Default for FontMetrics {
    fn default() -> Self {
        Self {
            font_face: Box::new(FONT_CACHE.fallback().clone()),
            size: DEFAULT_FONT_SIZE,
        }
    }
}
