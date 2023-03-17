use std::{
    cell::RefCell,
    ops::Deref,
    rc::{Rc, Weak},
};

pub use super::codegen::{DOMType, DOMTyped};

/// Smartpointer used for inheritance-objects.
/// Each [DOMPtr] contains a pointer to an object of type `T`.
/// `T` is either the actual type stored at the address or any
/// of its supertypes.
/// The internal objects are reference counted and inside a `RefCell`.
#[derive(Debug)]
pub struct DOMPtr<T: DOMTyped> {
    inner: Rc<RefCell<T>>,

    /// The actual type pointed to by inner.
    underlying_type: DOMType,
}

#[derive(Debug)]
pub struct WeakDOMPtr<T: DOMTyped> {
    inner: Weak<RefCell<T>>,

    /// The actual type pointed to by inner.
    underlying_type: DOMType,
}

impl<T: DOMTyped> Deref for DOMPtr<T> {
    type Target = Rc<RefCell<T>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: DOMTyped> Deref for WeakDOMPtr<T> {
    type Target = Weak<RefCell<T>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: DOMTyped> DOMPtr<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner: Rc::new(RefCell::new(inner)),
            underlying_type: T::as_type(),
        }
    }

    pub fn underlying_type(&self) -> DOMType {
        self.underlying_type
    }

    pub fn is_a<O: DOMTyped>(&self) -> bool {
        self.underlying_type.is_a(O::as_type())
    }

    /// Cast a object into another.
    /// One of the objects must inherit from another.
    ///
    /// # Panics
    /// This function panics if the types are incompatible
    pub fn into_type<O: DOMTyped>(self) -> DOMPtr<O> {
        assert!(self.is_a::<O>());
        unsafe { std::mem::transmute(self) }
    }

    pub fn downgrade(&self) -> WeakDOMPtr<T> {
        WeakDOMPtr {
            inner: Rc::downgrade(&self.inner),
            underlying_type: self.underlying_type,
        }
    }
}

impl<T: DOMTyped> WeakDOMPtr<T> {
    pub fn underlying_type(&self) -> DOMType {
        self.underlying_type
    }

    pub fn is_a<O: DOMTyped>(&self) -> bool {
        self.underlying_type.is_a(O::as_type())
    }

    pub fn upgrade(&self) -> Option<DOMPtr<T>> {
        self.inner.upgrade().map(|upgraded_ptr| DOMPtr {
            inner: upgraded_ptr,
            underlying_type: self.underlying_type,
        })
    }
}

impl<T: DOMTyped> Clone for DOMPtr<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            underlying_type: self.underlying_type,
        }
    }
}

impl<T: DOMTyped> Clone for WeakDOMPtr<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            underlying_type: self.underlying_type,
        }
    }
}
