use dom_derive::inherit;

use super::HtmlElement;

/// <https://html.spec.whatwg.org/multipage/embedded-content.html#the-img-element>
#[inherit(HtmlElement)]
pub struct HtmlImageElement {}

impl HtmlImageElement {
    pub fn new(html_element: HtmlElement) -> Self {
        Self {
            __parent: html_element,
        }
    }
}
