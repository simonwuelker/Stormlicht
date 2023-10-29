use std::mem;

use crate::dom::{self, RelativePosition};

#[derive(Clone, Debug)]
pub struct Selection {
    start: dom::BoundaryPoint,
    end: dom::BoundaryPoint,
    pub is_modifiable: bool,
    fixed_side: FixedSide,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FixedSide {
    Start,
    End,
}

impl Selection {
    #[inline]
    #[must_use]
    pub fn new(start: dom::BoundaryPoint, end: dom::BoundaryPoint) -> Self {
        Self {
            start,
            end,
            is_modifiable: true,
            fixed_side: FixedSide::Start,
        }
    }

    pub fn extend_to(&mut self, new_bound: dom::BoundaryPoint) {
        match self.fixed_side {
            FixedSide::Start => {
                if new_bound.position_relative_to(self.start()) == RelativePosition::Before {
                    self.end = mem::replace(&mut self.start, new_bound);
                    self.fixed_side = FixedSide::End;
                } else {
                    self.end = new_bound;
                }
            },
            FixedSide::End => {
                if new_bound.position_relative_to(self.end()) == RelativePosition::After {
                    self.start = mem::replace(&mut self.end, new_bound);
                    self.fixed_side = FixedSide::Start;
                } else {
                    self.start = new_bound;
                }
            },
        }
    }

    #[inline]
    #[must_use]
    pub fn start(&self) -> dom::BoundaryPoint {
        self.start.clone()
    }

    #[inline]
    #[must_use]
    pub fn end(&self) -> dom::BoundaryPoint {
        self.end.clone()
    }
}
