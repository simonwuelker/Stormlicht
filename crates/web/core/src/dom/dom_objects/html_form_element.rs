use dom_derive::inherit;

use super::HtmlElement;
use crate::display_tagname;

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

display_tagname!(HtmlFormElement, "form");
