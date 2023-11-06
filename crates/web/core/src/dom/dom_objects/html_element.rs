use dom_derive::inherit;

use super::Element;

/// <https://html.spec.whatwg.org/multipage/dom.html#htmlelement>
#[inherit(Element)]
pub struct HtmlElement {}

impl HtmlElement {
    pub fn new(element: Element) -> Self {
        Self { __parent: element }
    }
}
