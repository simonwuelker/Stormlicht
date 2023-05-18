//! <https://html.spec.whatwg.org/multipage/semantics.html#the-head-element>

use dom_derive::inherit;

use super::HTMLElement;

#[inherit(HTMLElement)]
pub struct HTMLHeadElement {}

impl HTMLHeadElement {
    pub fn new(html_element: HTMLElement) -> Self {
        Self {
            __parent: html_element,
        }
    }
}
