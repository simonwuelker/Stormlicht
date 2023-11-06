use dom_derive::inherit;

use super::HtmlElement;

/// <https://html.spec.whatwg.org/multipage/grouping-content.html#the-li-element>
#[inherit(HtmlElement)]
pub struct HtmlLiElement {}

impl HtmlLiElement {
    pub fn new(html_element: HtmlElement) -> Self {
        Self {
            __parent: html_element,
        }
    }
}
