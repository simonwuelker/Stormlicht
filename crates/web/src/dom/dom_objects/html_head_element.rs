use dom_derive::inherit;

use super::HtmlElement;

/// <https://html.spec.whatwg.org/multipage/semantics.html#the-head-element>
#[inherit(HtmlElement)]
pub struct HtmlHeadElement {}

impl HtmlHeadElement {
    pub fn new(html_element: HtmlElement) -> Self {
        Self {
            __parent: html_element,
        }
    }
}
