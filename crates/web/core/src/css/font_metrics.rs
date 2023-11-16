use font::Font;

use super::layout::Pixels;

pub const DEFAULT_FONT_SIZE: Pixels = Pixels(16.0);

#[derive(Clone, Debug)]
pub struct FontMetrics {
    pub font_face: Box<Font>,
    pub size: Pixels,
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
    pub fn new(size: Pixels) -> Self {
        Self {
            size,
            font_face: Box::default(),
        }
    }
}
