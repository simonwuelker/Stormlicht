use super::Node;
use dom_derive::inherit;

/// <https://dom.spec.whatwg.org/#interface-documenttype>
#[inherit(Node)]
pub struct DocumentType {
    name: String,
    public_id: String,
    system_id: String,
}

impl DocumentType {
    pub fn name_mut(&mut self) -> &mut String {
        &mut self.name
    }

    pub fn public_id_mut(&mut self) -> &mut String {
        &mut self.public_id
    }

    pub fn system_id_mut(&mut self) -> &mut String {
        &mut self.system_id
    }
}
