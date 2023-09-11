use crate::css::{layout::CSSPixels, FontMetrics};

#[derive(Clone, Debug)]
pub enum Command {
    Rect(RectCommand),
    Text(TextCommand),
}

#[derive(Clone, Copy, Debug)]
pub struct RectCommand {
    pub area: math::Rectangle<CSSPixels>,
    pub color: math::Color,
}

#[derive(Clone, Debug)]
pub struct TextCommand {
    pub position: math::Vec2D<CSSPixels>,
    pub text: String,
    pub font_metrics: FontMetrics,
    pub color: math::Color,
}
