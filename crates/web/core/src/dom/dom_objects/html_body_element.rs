use dom_derive::inherit;

use super::HtmlElement;

/// <https://html.spec.whatwg.org/multipage/sections.html#the-body-element>
#[inherit(HtmlElement)]
pub struct HtmlBodyElement {}

impl HtmlBodyElement {
    pub fn new(html_element: HtmlElement) -> Self {
        Self {
            __parent: html_element,
        }
    }
}
