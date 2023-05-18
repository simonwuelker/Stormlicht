//! <https://html.spec.whatwg.org/multipage/scripting.html#the-noscript-element>

use dom_derive::inherit;

use super::HTMLElement;

#[inherit(HTMLElement)]
pub struct HTMLNoscriptElement {}

impl HTMLNoscriptElement {
    pub fn new(html_element: HTMLElement) -> Self {
        Self {
            __parent: html_element,
        }
    }
}
