use dom_derive::inherit;

use super::HtmlElement;
use crate::display_tagname;

/// <https://html.spec.whatwg.org/multipage/semantics.html#the-html-element>
#[inherit(HtmlElement)]
pub struct HtmlHtmlElement {}

impl HtmlHtmlElement {
    pub fn new(html_element: HtmlElement) -> Self {
        Self {
            __parent: html_element,
        }
    }
}

display_tagname!(HtmlHtmlElement, "html");
