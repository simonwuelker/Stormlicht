use super::HTMLElement;
use crate::display_tagname;

use dom_derive::inherit;

/// <https://html.spec.whatwg.org/multipage/semantics.html#the-style-element>
#[inherit(HTMLElement)]
pub struct HTMLStyleElement {}

impl HTMLStyleElement {
    pub fn new(html_element: HTMLElement) -> Self {
        Self {
            __parent: html_element,
        }
    }
}

display_tagname!(HTMLStyleElement, "style");
