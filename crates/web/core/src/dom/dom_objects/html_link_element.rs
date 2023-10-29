use dom_derive::inherit;

use super::HTMLElement;
use crate::display_tagname;

/// <https://html.spec.whatwg.org/multipage/semantics.html#the-link-element>
#[inherit(HTMLElement)]
pub struct HTMLLinkElement {}

impl HTMLLinkElement {
    pub fn new(html_element: HTMLElement) -> Self {
        Self {
            __parent: html_element,
        }
    }
}

display_tagname!(HTMLLinkElement, "link");
