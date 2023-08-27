mod box_dimensions;
pub mod flow;
mod pixels;

pub use box_dimensions::BoxDimensions;
pub use pixels::CSSPixels;

#[derive(Clone, Copy, Debug)]
pub struct Sides<T> {
    pub top: T,
    pub right: T,
    pub bottom: T,
    pub left: T,
}

#[derive(Clone, Copy, Debug)]
pub struct UsedSizeAndMargins {
    pub width: CSSPixels,
    pub height: CSSPixels,
    pub margin: Sides<CSSPixels>,
}

pub type ContainingBlock = math::Rectangle;

pub trait Layout {
    /// Compute the box dimensions as specified in
    /// <https://drafts.csswg.org/css2/#Computing_widths_and_margins>
    /// and
    /// <https://drafts.csswg.org/css2/#Computing_heights_and_margins>
    fn compute_dimensions(&self, available_width: CSSPixels) -> UsedSizeAndMargins;
}
