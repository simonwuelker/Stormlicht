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

#[derive(Clone, Copy, Debug)]
pub enum Sizing {
    Exactly(u32),
    Grow(f32),
}
