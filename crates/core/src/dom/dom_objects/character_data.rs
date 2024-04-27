use super::Node;

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
