mod box_dimensions;
pub mod flow;
mod pixels;

pub use box_dimensions::BoxDimensions;
use math::{Rectangle, Vec2D};
pub use pixels::CSSPixels;

use std::ops;

#[derive(Clone, Copy, Debug)]
pub struct Sides<T> {
    pub top: T,
    pub right: T,
    pub bottom: T,
    pub left: T,
}

impl<T> Sides<T> {
    pub fn surround(&self, area: Rectangle<T>) -> Rectangle<T>
    where
        T: Copy,
        Vec2D<T>: ops::Add<Vec2D<T>, Output = Vec2D<T>> + ops::Sub<Vec2D<T>, Output = Vec2D<T>>,
    {
        Rectangle {
            top_left: area.top_left
                - Vec2D {
                    x: self.left,
                    y: self.top,
                },
            bottom_right: area.bottom_right
                + Vec2D {
                    x: self.right,
                    y: self.bottom,
                },
        }
    }
}
