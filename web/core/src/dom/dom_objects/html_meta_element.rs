use super::HTMLElement;
use crate::display_tagname;

use dom_derive::inherit;

/// <https://html.spec.whatwg.org/multipage/semantics.html#the-meta-element>
#[inherit(HTMLElement)]
pub struct HTMLMetaElement {
    name: String,
    http_equiv: String,
    content: String,
    media: String,
}

impl HTMLMetaElement {
    pub fn new(html_element: HTMLElement) -> Self {
        Self {
            __parent: html_element,
            name: String::new(),
            http_equiv: String::new(),
            content: String::new(),
            media: String::new(),
        }
    }
}

display_tagname!(HTMLMetaElement, "meta");
