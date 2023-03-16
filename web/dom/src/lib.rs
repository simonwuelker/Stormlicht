//! <https://dom.spec.whatwg.org/>

mod codegen;
pub mod dom_objects;
mod dom_type;

use std::ops::{Deref, DerefMut};

pub use codegen::{DOMType, DOMTyped};

/// Smartpointer used for inheritance-objects.
/// Each [DOMPtr] contains a pointer to an object of type `T`.
/// `T` is either the actual type stored at the address or any
/// of its supertypes.
#[derive(Debug)]
pub struct DOMPtr<T: DOMTyped> {
    /// We can't automatically drop this memory since we don't know how large
    /// the allocated section is (since T may be a supertype of the actually-stored type)
    inner: Box<T>,

    /// The actual type pointed to by inner.
    underlying_type: DOMType,
}

impl<T: DOMTyped> Deref for DOMPtr<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: DOMTyped> DerefMut for DOMPtr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T: DOMTyped> DOMPtr<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner: Box::new(inner),
            underlying_type: T::as_type(),
        }
    }

    pub fn underlying_type(&self) -> DOMType {
        self.underlying_type
    }

    pub fn is_a<O: DOMTyped>(&self) -> bool {
        self.underlying_type.is_a(O::as_type())
    }

    pub fn into_type<O: DOMTyped>(self) -> Option<DOMPtr<O>> {
        if self.is_a::<O>() {
            Some(unsafe { std::mem::transmute(self) })
        } else {
            None
        }
    }
}
