//! <https://html.spec.whatwg.org/multipage/scripting.html#the-script-element>

use dom_derive::inherit;

use super::HTMLElement;

#[inherit(HTMLElement)]
pub struct HTMLScriptElement {}

impl HTMLScriptElement {
    pub fn new(html_element: HTMLElement) -> Self {
        Self {
            __parent: html_element,
        }
    }
}
