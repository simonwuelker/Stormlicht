//! Thin abstraction over [std::ops::Range] with the aim of being easier to use.

#[derive(Clone, Copy, Debug)]
pub struct Range<T> {
    lower: RangeBound<T>,
    upper: RangeBound<T>,
}

#[derive(Clone, Copy, Debug)]
pub enum RangeBound<T> {
    Infinite,
    Finite(T),
}

impl<T: Copy> Range<T> {
    pub fn lower(&self) -> RangeBound<T> {
        self.lower
    }

    pub fn upper(&self) -> RangeBound<T> {
        self.upper
    }
}
