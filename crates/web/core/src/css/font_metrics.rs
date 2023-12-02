use font::Font;

use super::layout::Pixels;

pub const DEFAULT_FONT_SIZE: Pixels = Pixels(16.0);

#[derive(Clone, Debug)]
pub struct FontMetrics {
    pub font_face: Box<Font>,
    pub size: Pixels,
}
