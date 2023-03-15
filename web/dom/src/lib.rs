//! <https://dom.spec.whatwg.org/>

pub mod dom_objects;

// TODO: Placeholder type until the inheritance system is in place
pub struct DomPtr<T> {
    _inner: Box<T>,
}
