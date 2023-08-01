mod box_dimensions;
pub mod flow;
mod pixels;
mod styled_element;

use crate::{
    css::{StyleComputer, Stylesheet},
    dom::{dom_objects, DOMPtr},
};
pub use box_dimensions::BoxDimensions;
pub use pixels::CSSPixels;
pub use styled_element::StyledElement;

pub fn build_layout_tree_for_element(
    element: DOMPtr<dom_objects::Element>,
    stylesheets: &[Stylesheet],
    containing_block: ContainingBlock,
) {
    let style_computer = StyleComputer::new(stylesheets);
    let computed_style = style_computer.get_computed_style(element.clone().into_type());
    let display = computed_style.display();

    if display.is_none() {
        // Neither the element nor any of its descendants generate a box
        return;
    } else if display.is_contents() {
        // The element itself doesn't generate a box, but the contents do
        // as if the element itself didn't exist
        for child in element
            .into_type::<dom_objects::Node>()
            .borrow()
            .children()
            .iter()
            .filter_map(DOMPtr::try_into_type::<dom_objects::Element>)
        {
            build_layout_tree_for_element(child, stylesheets, containing_block)
        }
    } else {
        // FIXME: respect the actual display value
        // We currently assume everything to be flow layout
        let styled_element = StyledElement::new(element, &computed_style);
        let width = compute_used_widths_and_margins(styled_element, containing_block);
        _ = width;
    }
    todo!()
}

#[derive(Clone, Copy, Debug)]
pub struct UsedWidthsAndMargins {
    pub width: Option<CSSPixels>,
    pub margin_left: CSSPixels,
    pub margin_right: CSSPixels,
}

#[derive(Clone, Copy, Debug)]
pub enum ContainingBlock {
    Viewport(math::Rectangle),
    Block(math::Rectangle),
}

impl ContainingBlock {
    #[must_use]
    pub fn width(&self) -> f32 {
        match self {
            Self::Viewport(rect) => rect.width(),
            Self::Block(rect) => rect.width(),
        }
    }
}

/// <https://drafts.csswg.org/css2/#Computing_widths_and_margins>
fn compute_used_widths_and_margins(
    element: StyledElement,
    containing_block: ContainingBlock,
) -> UsedWidthsAndMargins {
    if element.style().display().is_inline() {
        if !element.element().borrow().is_replaced() {
            // 1. inline, non-replaced elements
            element.inline_width(containing_block)
        } else {
            // 2. inline, replaced elements
            element.inline_replaced_width(containing_block)
        }
    } else if element.style().display().is_block() {
        if !element.element().borrow().is_replaced() {
            // 3. https://drafts.csswg.org/css2/#blockwidth
            element.block_width(containing_block)
        } else {
            element.block_replaced_width(containing_block)
        }
    } else {
        todo!()
    }
}
