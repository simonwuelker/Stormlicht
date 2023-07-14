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

impl<T: PartialEq> PartialEq for Rectangle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.top_left == other.top_left && self.bottom_right == other.bottom_right
    }
}

impl<T: PartialOrd> Rectangle<T> {
    pub fn contains(&self, other: Self) -> bool {
        self.top_left.x <= other.top_left.x
            && self.top_left.y <= other.top_left.y
            && other.bottom_right.x <= self.bottom_right.x
            && other.bottom_right.y <= self.bottom_right.y
    }
}
