pub mod box_model;
mod styled_element;

use crate::{
    css::{values::LengthPercentage, StyleComputer, Stylesheet},
    dom::{dom_objects, DOMPtr},
};
pub use styled_element::StyledElement;

pub fn build_layout_tree_for_element(
    element: DOMPtr<dom_objects::Element>,
    stylesheets: &[Stylesheet],
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
            build_layout_tree_for_element(child, stylesheets)
        }
    } else {
        // FIXME: respect the actual display value
        // We currently assume everything to be flow layout
        let styled_element = StyledElement::new(element, &computed_style);
        let width = compute_used_widths_and_margins(styled_element);
        _ = width;
    }
    todo!()
}

#[derive(Clone, Copy, Debug)]
pub struct ComputedWidthsAndMargins {
    pub width: Option<LengthPercentage>,
    pub margin_left: LengthPercentage,
    pub margin_right: LengthPercentage,
}

/// <https://drafts.csswg.org/css2/#Computing_widths_and_margins>
fn compute_used_widths_and_margins(element: StyledElement) -> ComputedWidthsAndMargins {
    if element.style().display().is_inline() {
        if !element.element().borrow().is_replaced() {
            // 1. inline, non-replaced elements
            element.inline_width()
        } else {
            // 2. inline, replaced elements
            element.inline_replaced_width()
        }
    } else {
        todo!()
    }
}
