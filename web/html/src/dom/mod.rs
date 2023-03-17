//! <https://dom.spec.whatwg.org/>

mod codegen;
pub mod dom_objects;
mod dom_ptr;

pub use dom_ptr::{DOMPtr, WeakDOMPtr};
