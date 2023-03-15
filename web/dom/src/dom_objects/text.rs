use super::Node;
use dom_derive::inherit;

#[inherit(Node)]
pub struct Text {
    content: String,
}

impl Text {
    pub fn content(&self) -> &str {
        &self.content
    }
}
