mod box_tree;
pub mod flow;
mod formatting_context;
mod pixels;
mod replaced;

pub use box_tree::BoxTree;
pub use pixels::Pixels;

use math::{Rectangle, Vec2D};

use std::ops;
#[derive(Clone, Copy, Debug)]
pub struct Sides<T> {
    pub top: T,
    pub right: T,
    pub bottom: T,
    pub left: T,
}

impl<T> Sides<T>
where
    T: Copy,
    Vec2D<T>: ops::Add<Vec2D<T>, Output = Vec2D<T>> + ops::Sub<Vec2D<T>, Output = Vec2D<T>>,
{
    #[must_use]
    pub fn surround(&self, area: Rectangle<T>) -> Rectangle<T> {
        let top_left = area.top_left()
            - Vec2D {
                x: self.left,
                y: self.top,
            };
        let bottom_right = area.bottom_right()
            + Vec2D {
                x: self.right,
                y: self.bottom,
            };
        Rectangle::from_corners(top_left, bottom_right)
    }
}

impl<T> Sides<T> {
    #[must_use]
    pub fn map<F, R>(&self, f: F) -> Sides<R>
    where
        F: Fn(&T) -> R,
    {
        Sides {
            top: f(&self.top),
            right: f(&self.right),
            bottom: f(&self.bottom),
            left: f(&self.left),
        }
    }
}

impl<T: Copy> Sides<T> {
    pub const fn all(value: T) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }
}

impl<T, S> Sides<T>
where
    T: ops::Add<T, Output = S> + Copy,
{
    #[must_use]
    pub fn horizontal_sum(&self) -> S {
        self.left + self.right
    }

    #[must_use]
    pub fn vertical_sum(&self) -> S {
        self.top + self.bottom
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Size<T> {
    pub width: T,
    pub height: T,
}

impl<T> Size<T>
where
    T: ops::Add<Output = T> + ops::Sub<Output = T> + Copy,
{
    #[must_use]
    pub fn at_position(&self, position: math::Vec2D<T>) -> Rectangle<T> {
        Rectangle::from_position_and_size(position, self.width, self.height)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ContainingBlock {
    width: Pixels,

    /// The height of the containing block
    ///
    /// `Some` if the height is defined (for example, using the CSS "height" property)
    /// or `None` if the height depends on the content.
    height: Option<Pixels>,
}

impl ContainingBlock {
    #[inline]
    #[must_use]
    pub const fn new(width: Pixels) -> Self {
        Self {
            width,
            height: None,
        }
    }

    pub const fn with_height(mut self, height: Pixels) -> Self {
        self.height = Some(height);
        self
    }

    #[inline]
    #[must_use]
    pub const fn width(&self) -> Pixels {
        self.width
    }

    #[inline]
    #[must_use]
    pub const fn height(&self) -> Option<Pixels> {
        self.height
    }

    #[must_use]
    pub fn make_definite(&self, definite_height: Pixels) -> Size<Pixels> {
        Size {
            width: self.width,
            height: self.height.unwrap_or(definite_height),
        }
    }
}
