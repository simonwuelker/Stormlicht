use dom_derive::inherit;

use super::HTMLElement;
use crate::display_tagname;

/// <https://html.spec.whatwg.org/multipage/semantics.html#the-head-element>
#[inherit(HTMLElement)]
pub struct HTMLHeadElement {}

impl HTMLHeadElement {
    pub fn new(html_element: HTMLElement) -> Self {
        Self {
            __parent: html_element,
        }
    }
}

display_tagname!(HTMLHeadElement, "head");
