use dom_derive::inherit;

use super::Element;
use crate::display_string;

/// <https://html.spec.whatwg.org/multipage/dom.html#htmlelement>
#[inherit(Element)]
pub struct HtmlElement {}

display_string!(HtmlElement, "HTML");

impl HtmlElement {
    pub fn new(element: Element) -> Self {
        Self { __parent: element }
    }
}
