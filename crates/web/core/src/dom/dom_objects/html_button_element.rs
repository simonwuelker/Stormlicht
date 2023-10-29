use dom_derive::inherit;

use super::HTMLElement;
use crate::display_tagname;

/// <https://html.spec.whatwg.org/multipage/form-elements.html#the-button-element>
#[inherit(HTMLElement)]
pub struct HTMLButtonElement {}

impl HTMLButtonElement {
    pub fn new(html_element: HTMLElement) -> Self {
        Self {
            __parent: html_element,
        }
    }
}

display_tagname!(HTMLButtonElement, "button");
