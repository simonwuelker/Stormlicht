use dom_derive::inherit;

use super::HtmlElement;

/// <https://html.spec.whatwg.org/multipage/scripting.html#the-script-element>
#[inherit(HtmlElement)]
pub struct HtmlScriptElement {}

impl HtmlScriptElement {
    pub fn new(html_element: HtmlElement) -> Self {
        Self {
            __parent: html_element,
        }
    }
}
