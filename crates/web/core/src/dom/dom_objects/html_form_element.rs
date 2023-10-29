use dom_derive::inherit;

use super::HTMLElement;
use crate::display_tagname;

/// <https://html.spec.whatwg.org/multipage/forms.html#the-form-element>
#[inherit(HTMLElement)]
pub struct HTMLFormElement {}

impl HTMLFormElement {
    pub fn new(html_element: HTMLElement) -> Self {
        Self {
            __parent: html_element,
        }
    }
}

display_tagname!(HTMLFormElement, "form");
