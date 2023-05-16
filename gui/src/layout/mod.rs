mod container;
mod widget;

pub mod widgets;

pub use container::Container;
pub use widget::Widget;

#[derive(Debug)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Sizing {
    Exactly(u32),
    Grow(f32),
}

impl Default for Sizing {
    fn default() -> Self {
        Self::Grow(1.)
    }
}

impl PartialOrd for Sizing {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Self::Exactly(n), Self::Exactly(m)) => n.partial_cmp(m),
            (Self::Grow(_), Self::Exactly(_)) => Some(std::cmp::Ordering::Greater),
            (Self::Exactly(_), Self::Grow(_)) => Some(std::cmp::Ordering::Less),
            (Self::Grow(n), Self::Grow(m)) => n.partial_cmp(m),
        }
    }
}

impl Eq for Sizing {}

impl Ord for Sizing {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // our partial_cmp implementation can never return None
        // (maybe if we try something like Grow(NaN) but that shouldn't happen in practice)
        self.partial_cmp(other).unwrap()
    }
}
