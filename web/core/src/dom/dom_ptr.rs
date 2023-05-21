use std::{
    cell::RefCell,
    ops::Deref,
    rc::{Rc, Weak},
};

use super::codegen::{DOMType, DOMTyped};

/// Smartpointer used for inheritance-objects.
/// Each [DOMPtr] contains a pointer to an object of type `T`.
/// `T` is either the actual type stored at the address or any
/// of its supertypes.
/// The internal objects are reference counted and inside a `RefCell`.
pub struct DOMPtr<T: DOMTyped> {
    inner: Rc<RefCell<T>>,

    /// The actual type pointed to by inner.
    underlying_type: DOMType,
}

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

    /// Get the actual type pointed to by the [DOMPtr]
    pub fn underlying_type(&self) -> DOMType {
        self.underlying_type
    }

    /// Return true if the DOMPtr stores `O` or any of its subclasses
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

    /// Try to cast the object to another type and fail
    /// if the cast is invalid (ie the objects don't inherit from each other)
    pub fn try_into_type<O: DOMTyped>(&self) -> Option<DOMPtr<O>> {
        if self.is_a::<O>() {
            Some(DOMPtr::clone(self).into_type())
        } else {
            None
        }
    }

    /// Check if two [DOMPtr]'s point to the same object.
    /// This is the equivalent `ptr_eq` on [Rc](std::rc::Rc).
    /// Note that due to the constraints on [Rc], the two dom
    /// pointers must point to the same type.
    pub fn ptr_eq<U: DOMTyped>(&self, other: &DOMPtr<U>) -> bool {
        // We don't care about the type information,
        // only if the two DOMPtrs point to the same underlying object
        self.inner.as_ptr().cast::<U>() == other.as_ptr()
    }

    pub fn downgrade(&self) -> WeakDOMPtr<T> {
        WeakDOMPtr {
            inner: Rc::downgrade(&self.inner),
            underlying_type: self.underlying_type,
        }
    }
}

impl<T: DOMTyped> WeakDOMPtr<T> {
    /// Get the actual type pointed to by the [DOMPtr]
    pub fn underlying_type(&self) -> DOMType {
        self.underlying_type
    }

    /// Return true if the DOMPtr stores `O` or any of its subclasses
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
