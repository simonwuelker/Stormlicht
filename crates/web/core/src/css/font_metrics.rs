use font::Font;

use super::layout::CSSPixels;

pub const DEFAULT_FONT_SIZE: CSSPixels = CSSPixels(16.0);

#[derive(Clone, Debug)]
pub struct FontMetrics {
    pub font_face: Box<Font>,
    pub size: CSSPixels,
}

impl Default for FontMetrics {
    fn default() -> Self {
        Self {
            font_face: Box::default(),
            size: DEFAULT_FONT_SIZE,
        }
    }
}

impl FontMetrics {
    pub fn new(size: CSSPixels) -> Self {
        Self {
            size,
            font_face: Box::default(),
        }
    }
}
