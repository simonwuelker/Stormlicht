mod divider;
mod widget;

pub mod widgets;

pub use widget::Widget;
pub use divider::Divider;

#[derive(Debug)]
pub enum Orientation {
    Horizontal,
    Vertical,
}