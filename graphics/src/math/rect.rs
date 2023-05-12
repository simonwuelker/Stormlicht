use super::Vec2D;

#[derive(Clone, Copy, Debug, Default)]
pub struct Rectangle<T = f32> {
    pub top_left: Vec2D<T>,
    pub bottom_right: Vec2D<T>,
}
