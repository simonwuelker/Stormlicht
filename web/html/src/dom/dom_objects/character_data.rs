use super::Node;
use crate::dom::DOMDisplay;
use std::fmt;

use dom_derive::inherit;

/// <https://dom.spec.whatwg.org/#characterdata>
#[inherit(Node)]
pub struct CharacterData {
    content: String,
}

impl CharacterData {
    pub fn content_mut(&mut self) -> &mut String {
        &mut self.content
    }
}

impl DOMDisplay for CharacterData {
    fn format(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "CDATA({:?})", self.content)
    }
}
