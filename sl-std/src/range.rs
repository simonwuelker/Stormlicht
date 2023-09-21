#[derive(Clone, Copy, Debug)]
pub struct Range<I> {
    start: I,
    end: I,
}

impl<I: Copy> Range<I> {
    #[inline]
    #[must_use]
    pub fn start(&self) -> I {
        self.start
    }

    #[inline]
    #[must_use]
    pub fn end(&self) -> I {
        self.end
    }
}

impl<I: PartialOrd> Range<I> {
    #[inline]
    #[must_use]
    pub fn new(start: I, end: I) -> Self {
        debug_assert!(start <= end);
        Self { start, end }
    }
}

impl<I: PartialEq> Range<I> {
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

impl<I: Ord + Copy> Range<I> {
    /// Clamp the range to a lower bound
    ///
    /// It is guaranteed that both the start and the end of the returned range
    /// are larger than `lower_bound`
    #[inline]
    #[must_use]
    pub fn clamp_start(&self, lower_bound: I) -> Self {
        Self {
            start: self.start.max(lower_bound),
            end: self.end.max(lower_bound),
        }
    }

    /// Clamp the range to an upper bound
    ///
    /// It is guaranteed that both the start and the end of the returned range
    /// are smaller than `upper_bound`
    #[inline]
    #[must_use]
    pub fn clamp_end(&self, upper_bound: I) -> Self {
        Self {
            start: self.start.min(upper_bound),
            end: self.end.min(upper_bound),
        }
    }

    #[inline]
    #[must_use]
    pub fn intersection(&self, other: Self) -> Option<Self> {
        if other.end() <= self.start() || self.end() <= other.start() {
            None
        } else {
            Some(Self {
                start: self.start().max(other.start()),
                end: self.end().min(other.end()),
            })
        }
    }
}
