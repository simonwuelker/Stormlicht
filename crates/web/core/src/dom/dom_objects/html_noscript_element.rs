use dom_derive::inherit;

use super::HtmlElement;
use crate::display_tagname;

/// <https://html.spec.whatwg.org/multipage/scripting.html#the-noscript-element>
#[inherit(HtmlElement)]
pub struct HtmlNoscriptElement {}

impl HtmlNoscriptElement {
    pub fn new(html_element: HtmlElement) -> Self {
        Self {
            __parent: html_element,
        }
    }
}

display_tagname!(HtmlNoscriptElement, "noscript");
