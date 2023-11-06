use super::HtmlElement;
use crate::display_tagname;

use dom_derive::inherit;

/// <https://html.spec.whatwg.org/multipage/semantics.html#the-style-element>
#[inherit(HtmlElement)]
pub struct HtmlStyleElement {}

impl HtmlStyleElement {
    pub fn new(html_element: HtmlElement) -> Self {
        Self {
            __parent: html_element,
        }
    }
}

display_tagname!(HtmlStyleElement, "style");
