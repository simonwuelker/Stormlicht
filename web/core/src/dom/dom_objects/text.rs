use super::Node;
use dom_derive::inherit;
use std::fmt;

/// <https://dom.spec.whatwg.org/#interface-text>
#[inherit(Node)]
pub struct Text {
    content: String,
}

impl Text {
    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn content_mut(&mut self) -> &mut String {
        &mut self.content
    }
}

impl crate::dom::DOMDisplay for Text {
    fn format<W: fmt::Write>(&self, writer: &mut W) -> fmt::Result {
        self.format_text(writer, &self.content.escape_default().collect::<String>())
    }
}
