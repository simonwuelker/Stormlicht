use super::HtmlElement;
use crate::display_tagname;

use dom_derive::inherit;

/// <https://html.spec.whatwg.org/multipage/semantics.html#the-meta-element>
#[inherit(HtmlElement)]
pub struct HtmlMetaElement {
    name: String,
    http_equiv: String,
    content: String,
    media: String,
}

impl HtmlMetaElement {
    pub fn new(html_element: HtmlElement) -> Self {
        Self {
            __parent: html_element,
            name: String::new(),
            http_equiv: String::new(),
            content: String::new(),
            media: String::new(),
        }
    }
}

display_tagname!(HtmlMetaElement, "meta");
