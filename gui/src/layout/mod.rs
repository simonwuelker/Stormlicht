mod divider;
mod widget;

pub mod widgets;

pub use divider::Divider;
pub use widget::Widget;

#[derive(Debug)]
pub enum Orientation {
    Horizontal,
    Vertical,
}
