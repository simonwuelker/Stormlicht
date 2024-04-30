use dom_derive::inherit;

use super::HtmlElement;

/// <https://html.spec.whatwg.org/multipage/grouping-content.html#the-dd-element>
#[inherit(HtmlElement)]
pub struct HtmlDdElement {}

impl HtmlDdElement {
    pub fn new(html_element: HtmlElement) -> Self {
        Self {
            __parent: html_element,
        }
    }
}
