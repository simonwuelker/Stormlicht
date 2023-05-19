//! <https://html.spec.whatwg.org/multipage/sections.html#the-body-element>

use dom_derive::inherit;

use super::HTMLElement;
use crate::display_tagname;

#[inherit(HTMLElement)]
pub struct HTMLBodyElement {}

impl HTMLBodyElement {
    pub fn new(html_element: HTMLElement) -> Self {
        Self {
            __parent: html_element,
        }
    }
}

display_tagname!(HTMLBodyElement, "body");
