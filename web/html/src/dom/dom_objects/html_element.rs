//! <https://html.spec.whatwg.org/multipage/dom.html#htmlelement>

use dom_derive::inherit;

use super::Element;

#[inherit(Element)]
pub struct HTMLElement {}

impl HTMLElement {
    pub fn new(element: Element) -> Self {
        Self { __parent: element }
    }
}
