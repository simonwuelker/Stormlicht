use super::Node;
use dom_derive::inherit;
use string_interner::InternedString;

/// <https://dom.spec.whatwg.org/#interface-documenttype>
#[inherit(Node)]
pub struct DocumentType {
    name: InternedString,
    public_id: InternedString,
    system_id: InternedString,
}

impl DocumentType {
    pub fn set_name(&mut self, name: InternedString) {
        self.name = name;
    }

    pub fn set_public_id(&mut self, public_id: InternedString) {
        self.public_id = public_id;
    }

    pub fn set_system_id(&mut self, system_id: InternedString) {
        self.system_id = system_id;
    }
}
