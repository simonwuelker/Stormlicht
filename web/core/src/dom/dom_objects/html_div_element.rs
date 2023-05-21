use dom_derive::inherit;

use super::HTMLElement;
use crate::display_tagname;

/// <https://html.spec.whatwg.org/multipage/grouping-content.html#the-div-element>
#[inherit(HTMLElement)]
pub struct HTMLDivElement {}

impl HTMLDivElement {
    pub fn new(html_element: HTMLElement) -> Self {
        Self {
            __parent: html_element,
        }
    }
}

display_tagname!(HTMLDivElement, "div");
