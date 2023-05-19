use super::HTMLElement;
use crate::display_tagname;

use dom_derive::inherit;

/// <https://html.spec.whatwg.org/multipage/semantics.html#the-title-element>
#[inherit(HTMLElement)]
pub struct HTMLTitleElement {
    text: String,
}

impl HTMLTitleElement {
    pub fn new(html_element: HTMLElement) -> Self {
        Self {
            __parent: html_element,
            text: String::new(),
        }
    }
}

display_tagname!(HTMLTitleElement, "title");
