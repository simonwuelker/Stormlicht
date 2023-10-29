use dom_derive::inherit;

use super::Node;
use crate::display_string;

/// <https://dom.spec.whatwg.org/#interface-document>
#[inherit(Node)]
pub struct Document {
    charset: String,
}

display_string!(Document, "DOCUMENT");

impl Document {
    pub fn charset(&self) -> &str {
        &self.charset
    }
}
