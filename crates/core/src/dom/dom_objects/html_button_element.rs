use dom_derive::inherit;

use super::HtmlElement;

/// <https://html.spec.whatwg.org/multipage/form-elements.html#the-button-element>
#[inherit(HtmlElement)]
pub struct HtmlButtonElement {}

impl HtmlButtonElement {
    pub fn new(html_element: HtmlElement) -> Self {
        Self {
            __parent: html_element,
        }
    }
}
