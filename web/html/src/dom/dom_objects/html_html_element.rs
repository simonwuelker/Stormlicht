//! <https://html.spec.whatwg.org/multipage/semantics.html#the-html-element>

use dom_derive::inherit;

use super::HTMLElement;

#[inherit(HTMLElement)]
pub struct HTMLHtmlElement {}

impl HTMLHtmlElement {
    pub fn new(html_element: HTMLElement) -> Self {
        Self {
            __parent: html_element,
        }
    }
}
