use dom_derive::inherit;

use super::HtmlElement;

/// <https://html.spec.whatwg.org/multipage/forms.html#the-form-element>
#[inherit(HtmlElement)]
pub struct HtmlFormElement {}

impl HtmlFormElement {
    pub fn new(html_element: HtmlElement) -> Self {
        Self {
            __parent: html_element,
        }
    }
}
