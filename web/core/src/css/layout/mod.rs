mod box_dimensions;
pub mod flow;
mod pixels;
mod styled_element;

pub use box_dimensions::BoxDimensions;
pub use pixels::CSSPixels;
// pub use styled_element::StyledElement;

#[derive(Clone, Copy, Debug)]
pub struct Sides<T> {
    pub top: T,
    pub right: T,
    pub bottom: T,
    pub left: T,
}

// /// Describes a box size and the margins on either side of the box.
// ///
// /// This struct is used in both vertical and horizontal contexts.
// ///
// /// ## Horizontal
// /// In a horizontal context, `content` is the elements width,
// /// `margin_before` is the margin to the left side of the element and
// /// `margin_after` is the margin to the right side of the element.
// ///
// /// ## Vertical
// /// In a vertical context, `content` is the elements height,
// /// `margin_before` is the margin on top of the element and
// /// `margin_after` is the margin below the element.
// #[derive(Clone, Copy, Debug)]
// pub struct SizeAndMargins {
//     pub content: CSSPixels,
//     pub margin_before: CSSPixels,
//     pub margin_after: CSSPixels,
// }

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
