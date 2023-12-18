use image::Texture;

use crate::css::{layout::Pixels, FontMetrics};

#[derive(Clone, Debug)]
pub enum Command {
    Rect(RectCommand),
    Text(TextCommand),
    Image(ImageCommand),
}

#[derive(Clone, Copy, Debug)]
pub struct RectCommand {
    pub area: math::Rectangle<Pixels>,
    pub color: math::Color,
}

#[derive(Clone, Debug)]
pub struct TextCommand {
    pub position: math::Vec2D<Pixels>,
    pub text: String,
    pub font_metrics: FontMetrics,
    pub color: math::Color,
}

#[derive(Clone, Debug)]
pub struct ImageCommand {
    pub area: math::Rectangle<Pixels>,
    pub texture: Texture<u32>,
}
