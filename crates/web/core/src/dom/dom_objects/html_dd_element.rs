use dom_derive::inherit;

use super::HTMLElement;
use crate::display_tagname;

/// <https://html.spec.whatwg.org/multipage/grouping-content.html#the-dd-element>
#[inherit(HTMLElement)]
pub struct HTMLDdElement {}

impl HTMLDdElement {
    pub fn new(html_element: HTMLElement) -> Self {
        Self {
            __parent: html_element,
        }
    }
}

display_tagname!(HTMLDdElement, "dd");
