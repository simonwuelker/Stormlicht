use dom_derive::inherit;

use super::Node;

/// <https://dom.spec.whatwg.org/#interface-document>
#[inherit(Node)]
pub struct Document {
    charset: String,
}

impl Document {
    pub fn charset(&self) -> &str {
        &self.charset
    }
}
