use super::Vec2D;

use std::ops;

#[derive(Clone, Copy, Debug, Default)]
pub struct Rectangle<T = f32> {
    pub top_left: Vec2D<T>,
    pub bottom_right: Vec2D<T>,
}

impl<T> Rectangle<T>
where
    T: Copy,
{
    pub const fn top_left(&self) -> Vec2D<T> {
        self.top_left
    }

    pub const fn top_right(&self) -> Vec2D<T> {
        Vec2D {
            x: self.bottom_right.x,
            y: self.top_left.y,
        }
    }

    pub const fn bottom_left(&self) -> Vec2D<T> {
        Vec2D {
            x: self.top_left.x,
            y: self.bottom_right.y,
        }
    }

    pub const fn bottom_right(&self) -> Vec2D<T> {
        self.bottom_right
    }
}

impl<T> Rectangle<T>
where
    T: ops::Sub<Output = T> + Copy,
{
    pub fn width(&self) -> T {
        self.bottom_right.x - self.top_left.x
    }

    pub fn height(&self) -> T {
        self.bottom_right.y - self.top_left.y
    }
}

impl Rectangle<f32> {
    /// Create a pixel-aligned rectangle containing `self`
    ///
    /// The aligned rectangle is chosen to be as small as possible,
    /// but is guaranteed to contain `self` in its entirety.
    pub fn snap_to_grid(&self) -> Rectangle<usize> {
        Rectangle {
            top_left: self.top_left.map(|value| value.floor() as usize),
            bottom_right: self.bottom_right.map(|value| value.ceil() as usize),
        }
    }
}

impl<T: PartialEq> PartialEq for Rectangle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.top_left == other.top_left && self.bottom_right == other.bottom_right
    }
}

impl<T: PartialOrd + Copy> Rectangle<T> {
    #[inline]
    #[must_use]
    pub fn contains(&self, other: Self) -> bool {
        self.contains_point(other.top_left) && self.contains_point(other.bottom_right)
    }

    #[inline]
    #[must_use]
    pub fn contains_point(&self, point: Vec2D<T>) -> bool {
        (self.top_left.x..=self.bottom_right.x).contains(&point.x)
            && (self.top_left.y..=self.bottom_right.y).contains(&point.y)
    }

    #[inline]
    pub fn grow_to_contain(&mut self, other: Self) {
        // Like Ord::min/Ord::max except they only require T to implement
        // PartialOrd, not Ord

        let min = |a, b| {
            if a < b {
                a
            } else {
                b
            }
        };
        let max = |a, b| {
            if a < b {
                b
            } else {
                a
            }
        };
        self.top_left.x = min(self.top_left.x, other.top_left.x);
        self.top_left.y = min(self.top_left.y, other.top_left.y);
        self.bottom_right.x = max(self.bottom_right.x, other.bottom_right.x);
        self.bottom_right.y = max(self.bottom_right.y, other.bottom_right.y);
    }
}
