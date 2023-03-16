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
}
