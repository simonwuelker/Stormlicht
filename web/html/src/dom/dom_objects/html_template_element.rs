//! <https://html.spec.whatwg.org/multipage/scripting.html#the-template-element>

use dom_derive::inherit;

use super::HTMLElement;

#[inherit(HTMLElement)]
pub struct HTMLTemplateElement {}

impl HTMLTemplateElement {
    pub fn new(html_element: HTMLElement) -> Self {
        Self {
            __parent: html_element,
        }
    }
}
