//! <https://html.spec.whatwg.org/multipage/dom.html#htmlelement>

use dom_derive::inherit;

use super::Element;
use crate::display_string;

#[inherit(Element)]
pub struct HTMLElement {}

display_string!(HTMLElement, "HTML");

impl HTMLElement {
    pub fn new(element: Element) -> Self {
        Self { __parent: element }
    }
}
