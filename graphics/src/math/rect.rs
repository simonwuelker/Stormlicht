use std::ops::{
    Bound, Range as StdRange, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive,
};

use super::Vec2D;

/// Thin abstraction over [std::ops::Range] with the aim of being easier to use.
#[derive(Clone, Copy, Debug)]
pub struct Range<T> {
    lower: Bound<T>,
    upper: Bound<T>,
}

impl<T: PartialOrd + Copy> Range<T> {
    /// Check if a given value is contained within the range
    fn contains(&self, value: T) -> bool {
        match (self.lower, self.upper) {
            (Bound::Unbounded, Bound::Unbounded) => true,
            (Bound::Unbounded, Bound::Included(max)) => value <= max,
            (Bound::Unbounded, Bound::Excluded(max)) => value < max,
            (Bound::Included(min), Bound::Unbounded) => min <= value,
            (Bound::Included(min), Bound::Included(max)) => min <= value && value <= max,
            (Bound::Included(min), Bound::Excluded(max)) => min <= value && value < max,
            (Bound::Excluded(min), Bound::Unbounded) => min < value,
            (Bound::Excluded(min), Bound::Included(max)) => min < value && value <= max,
            (Bound::Excluded(min), Bound::Excluded(max)) => min < value && value < max,
        }
    }
}

impl<T: Copy> Range<T> {
    /// The lower bound of the range
    pub fn lower(&self) -> Bound<T> {
        self.lower
    }

    /// The upper bound of the range
    pub fn upper(&self) -> Bound<T> {
        self.upper
    }
}

impl<T> From<StdRange<T>> for Range<T> {
    fn from(value: StdRange<T>) -> Self {
        Self {
            lower: Bound::Included(value.start),
            upper: Bound::Excluded(value.end),
        }
    }
}

impl<T> From<RangeFrom<T>> for Range<T> {
    fn from(value: RangeFrom<T>) -> Self {
        Self {
            lower: Bound::Included(value.start),
            upper: Bound::Unbounded,
        }
    }
}

impl<T> From<RangeFull> for Range<T> {
    fn from(_: RangeFull) -> Self {
        Self {
            lower: Bound::Unbounded,
            upper: Bound::Unbounded,
        }
    }
}

impl<T> From<RangeInclusive<T>> for Range<T> {
    fn from(value: RangeInclusive<T>) -> Self {
        let (start, end) = value.into_inner();
        Self {
            lower: Bound::Included(start),
            upper: Bound::Included(end),
        }
    }
}

impl<T> From<RangeTo<T>> for Range<T> {
    fn from(value: RangeTo<T>) -> Self {
        Self {
            lower: Bound::Unbounded,
            upper: Bound::Excluded(value.end),
        }
    }
}

impl<T> From<RangeToInclusive<T>> for Range<T> {
    fn from(value: RangeToInclusive<T>) -> Self {
        Self {
            lower: Bound::Unbounded,
            upper: Bound::Included(value.end),
        }
    }
}

/// 2 dimensional rectangle.
///
/// You know what a rectangle is.
pub struct Rectangle {
    horizontal_range: Range<f32>,
    vertical_range: Range<f32>,
}

impl Rectangle {
    pub fn new<H, V>(horizontal_range: H, vertical_range: V) -> Self
    where
        H: Into<Range<f32>>,
        V: Into<Range<f32>>,
    {
        Self {
            horizontal_range: horizontal_range.into(),
            vertical_range: vertical_range.into(),
        }
    }

    pub fn contains_vertical(&self, value: Vec2D) -> bool {
        self.vertical_range.contains(value.y)
    }

    pub fn contains_horizontal(&self, value: Vec2D) -> bool {
        self.horizontal_range.contains(value.x)
    }

    pub fn contains(&self, value: Vec2D) -> bool {
        self.contains_horizontal(value) && self.contains_vertical(value)
    }
}
