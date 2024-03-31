use dom_derive::inherit;

use super::HtmlElement;

/// <https://html.spec.whatwg.org/multipage/tables.html#the-table-element>
#[inherit(HtmlElement)]
pub struct HtmlTableElement {}

impl HtmlTableElement {
    pub fn new(html_element: HtmlElement) -> Self {
        Self {
            __parent: html_element,
        }
    }
}
