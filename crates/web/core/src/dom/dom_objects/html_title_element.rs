use super::HtmlElement;
use crate::display_tagname;

use dom_derive::inherit;

/// <https://html.spec.whatwg.org/multipage/semantics.html#the-title-element>
#[inherit(HtmlElement)]
pub struct HtmlTitleElement {
    text: String,
}

impl HtmlTitleElement {
    pub fn new(html_element: HtmlElement) -> Self {
        Self {
            __parent: html_element,
            text: String::new(),
        }
    }
}

display_tagname!(HtmlTitleElement, "title");
