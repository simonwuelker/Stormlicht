use crate::{
    css::computed_style::ComputedStyle,
    dom::{dom_objects, DomPtr},
};

use super::replaced::ReplacedElement;

/// Describes what the visual content of an element is
///
/// Usually, this is simply the subtree of the element, converted to a box
/// tree and laid out. But in the case of replaced elements, the content has
/// little to nothing to do with the DOM itself.
#[derive(Clone, Debug)]
pub enum Content {
    Element,
    Replaced(ReplacedElement),
}

impl Content {
    #[must_use]
    pub fn for_element(element: DomPtr<dom_objects::Element>, style: ComputedStyle) -> Self {
        if let Some(replaced_content) = ReplacedElement::try_from(element, style) {
            Self::Replaced(replaced_content)
        } else {
            Self::Element
        }
    }
}
