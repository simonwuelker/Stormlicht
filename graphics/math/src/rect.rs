use super::Vec2D;

#[derive(Clone, Copy, Debug, Default)]
pub struct Rectangle<T = f32> {
    pub top_left: Vec2D<T>,
    pub bottom_right: Vec2D<T>,
}

impl<T: std::ops::Sub<Output = T> + Copy> Rectangle<T> {
    pub fn width(&self) -> T {
        self.bottom_right.x - self.top_left.x
    }

    pub fn height(&self) -> T {
        self.bottom_right.y - self.top_left.y
    }
}

impl Rectangle<f32> {
    pub fn round_to_grid(&self) -> Rectangle<usize> {
        Rectangle {
            top_left: self.top_left.round_to_grid(),
            bottom_right: self.bottom_right.round_to_grid(),
        }
    }
}
