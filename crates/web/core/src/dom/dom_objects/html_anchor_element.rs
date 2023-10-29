use dom_derive::inherit;

use super::HTMLElement;
use crate::display_tagname;

/// <https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-a-element>
#[inherit(HTMLElement)]
pub struct HTMLAnchorElement {}

impl HTMLAnchorElement {
    pub fn new(html_element: HTMLElement) -> Self {
        Self {
            __parent: html_element,
        }
    }
}

display_tagname!(HTMLAnchorElement, "a");
