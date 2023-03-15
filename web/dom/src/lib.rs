//! <https://dom.spec.whatwg.org/>

mod codegen;
pub mod dom_objects;
mod dom_type;

// TODO: Placeholder type until the inheritance system is in place
pub struct DomPtr<T> {
    _inner: Box<T>,
}
