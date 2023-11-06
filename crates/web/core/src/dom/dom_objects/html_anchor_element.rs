use dom_derive::inherit;

use super::HtmlElement;

/// <https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-a-element>
#[inherit(HtmlElement)]
pub struct HtmlAnchorElement {}

impl HtmlAnchorElement {
    pub fn new(html_element: HtmlElement) -> Self {
        Self {
            __parent: html_element,
        }
    }
}
