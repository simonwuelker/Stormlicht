use dom_derive::inherit;

use super::HtmlElement;

/// <https://html.spec.whatwg.org/multipage/scripting.html#the-script-element>
#[inherit(HtmlElement)]
pub struct HtmlScriptElement {
    /// <https://html.spec.whatwg.org/multipage/scripting.html#ready-to-be-parser-executed>
    ready_to_be_parser_executed: bool,
}

impl HtmlScriptElement {
    pub fn new(html_element: HtmlElement) -> Self {
        Self {
            ready_to_be_parser_executed: false,
            __parent: html_element,
        }
    }

    pub fn set_ready_to_be_parser_executed(&mut self) {
        self.ready_to_be_parser_executed = true;
    }

    #[must_use]
    pub fn is_ready_to_be_parser_executed(&self) -> bool {
        self.ready_to_be_parser_executed
    }
}
