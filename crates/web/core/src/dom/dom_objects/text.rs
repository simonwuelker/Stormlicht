use super::Node;
use dom_derive::inherit;

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
