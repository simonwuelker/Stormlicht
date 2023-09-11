use font::Font;

use crate::css::layout::CSSPixels;

#[derive(Clone, Copy, Debug)]
pub enum Command<'box_tree, 'font> {
    Rect(RectCommand),
    Text(TextCommand<'box_tree, 'font>),
}

#[derive(Clone, Copy, Debug)]
pub struct RectCommand {
    pub area: math::Rectangle<CSSPixels>,
    pub color: math::Color,
}

#[derive(Clone, Copy, Debug)]
pub struct TextCommand<'box_tree, 'font> {
    pub position: math::Vec2D<CSSPixels>,
    pub text: &'box_tree str,
    pub font: &'font Font,
}
