use dom_derive::inherit;

use super::HTMLElement;
use crate::display_tagname;

/// <https://html.spec.whatwg.org/multipage/grouping-content.html#the-li-element>
#[inherit(HTMLElement)]
pub struct HTMLLIElement {}

impl HTMLLIElement {
    pub fn new(html_element: HTMLElement) -> Self {
        Self {
            __parent: html_element,
        }
    }
}

display_tagname!(HTMLLIElement, "li");
