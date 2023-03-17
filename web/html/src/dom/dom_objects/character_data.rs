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

// A comment just adds an extra constructor which we don't care about
// since we dont support scripting anyways.
// So to simplify the code, a comment is simply a type alias
pub type Comment = CharacterData;
