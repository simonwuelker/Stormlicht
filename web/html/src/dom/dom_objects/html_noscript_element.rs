//! <https://html.spec.whatwg.org/multipage/scripting.html#the-noscript-element>

use dom_derive::inherit;

use super::HTMLElement;
use crate::display_tagname;

#[inherit(HTMLElement)]
pub struct HTMLNoscriptElement {}

impl HTMLNoscriptElement {
    pub fn new(html_element: HTMLElement) -> Self {
        Self {
            __parent: html_element,
        }
    }
}

display_tagname!(HTMLNoscriptElement, "noscript");
