//! <https://dom.spec.whatwg.org/>

mod codegen;
pub mod dom_objects;
mod dom_type;

use codegen::GlobalInheritanceObject;

// TODO: Placeholder type until the inheritance system is in place
pub struct DOMPtr<T> {
    _inner: Box<T>,
    underlying_type: GlobalInheritanceObject,
}

impl<T> DOMPtr<T> {
    pub fn underlying_type(&self) -> GlobalInheritanceObject {
        self.underlying_type
    }
}
