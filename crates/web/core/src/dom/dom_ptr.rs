use std::{
    cell::RefCell,
    fmt::Write,
    ops::Deref,
    sync::{Arc, Weak},
};

use crate::TreeDebug;

use super::{
    codegen::{DomType, DomTyped},
    dom_objects, IsA,
};

/// Smartpointer used for inheritance-objects.
/// Each [DomPtr] contains a pointer to an object of type `T`.
/// `T` is either the actual type stored at the address or any
/// of its supertypes.
/// The internal objects are reference counted and inside a `RefCell`.
pub struct DomPtr<T: DomTyped> {
    inner: Arc<RefCell<T>>,

    /// The actual type pointed to by inner.
    underlying_type: DomType,
}

pub struct WeakDomPtr<T: DomTyped> {
    inner: Weak<RefCell<T>>,

    /// The actual type pointed to by inner.
    underlying_type: DomType,
}

impl<T: DomTyped> Deref for DomPtr<T> {
    type Target = Arc<RefCell<T>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: DomTyped> Deref for WeakDomPtr<T> {
    type Target = Weak<RefCell<T>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: DomTyped> DomPtr<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner: Arc::new(RefCell::new(inner)),
            underlying_type: T::as_type(),
        }
    }

    /// Get the actual type pointed to by the [DomPtr]
    pub fn underlying_type(&self) -> DomType {
        self.underlying_type
    }

    /// Return true if the [DomPtr] stores `O` or any of its subclasses
    pub fn is_a<O: DomTyped>(&self) -> bool {
        self.underlying_type.is_a(O::as_type())
    }

    /// Cast a object into another.
    /// One of the objects must inherit from another.
    ///
    /// # Panics
    /// This function panics if the types are incompatible
    pub fn into_type<O: DomTyped>(self) -> DomPtr<O> {
        assert!(
            self.is_a::<O>(),
            "Cannot cast object of type \"{:?}\" into object of type \"{:?}\"",
            T::as_type(),
            O::as_type()
        );

        unsafe { self.cast_unchecked() }
    }

    /// Cast a object into an instance of one of its parent classes
    pub fn upcast<O: DomTyped>(self) -> DomPtr<O>
    where
        T: IsA<O>,
    {
        debug_assert!(self.is_a::<O>());

        // SAFETY: IsA is only implemented by the build script
        unsafe { self.cast_unchecked() }
    }

    unsafe fn cast_unchecked<O: DomTyped>(self) -> DomPtr<O> {
        std::mem::transmute(self)
    }

    /// Try to cast the object to another type and fail
    /// if the cast is invalid (ie the objects don't inherit from each other)
    pub fn try_into_type<O: DomTyped>(&self) -> Option<DomPtr<O>> {
        if self.is_a::<O>() {
            let result = unsafe { self.clone().cast_unchecked() };
            Some(result)
        } else {
            None
        }
    }

    /// Check if two [DomPtr]'s point to the same object.
    /// This is the equivalent `ptr_eq` on [Rc](std::rc::Rc).
    /// Note that due to the constraints on [Rc], the two dom
    /// pointers must point to the same type.
    pub fn ptr_eq<U: DomTyped>(&self, other: &DomPtr<U>) -> bool {
        // We don't care about the type information,
        // only if the two DOMPtrs point to the same underlying object
        self.inner.as_ptr().cast::<U>() == other.as_ptr()
    }

    pub fn downgrade(&self) -> WeakDomPtr<T> {
        WeakDomPtr {
            inner: Arc::downgrade(&self.inner),
            underlying_type: self.underlying_type,
        }
    }
}

impl<T: DomTyped> WeakDomPtr<T> {
    /// Get the actual type pointed to by the [DomPtr]
    pub fn underlying_type(&self) -> DomType {
        self.underlying_type
    }

    /// Return true if the [DomPtr] stores `O` or any of its subclasses
    pub fn is_a<O: DomTyped>(&self) -> bool {
        self.underlying_type.is_a(O::as_type())
    }

    pub fn upgrade(&self) -> Option<DomPtr<T>> {
        self.inner.upgrade().map(|upgraded_ptr| DomPtr {
            inner: upgraded_ptr,
            underlying_type: self.underlying_type,
        })
    }
}

impl<T: DomTyped> Clone for DomPtr<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            underlying_type: self.underlying_type,
        }
    }
}

impl<T: DomTyped> Clone for WeakDomPtr<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            underlying_type: self.underlying_type,
        }
    }
}

impl<T> TreeDebug for DomPtr<T>
where
    T: DomTyped,
{
    fn tree_fmt(&self, formatter: &mut crate::TreeFormatter<'_, '_>) -> std::fmt::Result {
        if let Some(node) = self.try_into_type::<dom_objects::Node>() {
            formatter.indent()?;

            if let Some(text) = self.try_into_type::<dom_objects::Text>() {
                formatter.write_text(text.borrow().content(), "\"", "\"")?;
            } else if let Some(comment) = self.try_into_type::<dom_objects::Comment>() {
                formatter.write_text(comment.borrow().comment_data(), "<!--", "-->")?;
            } else if let Some(element) = self.try_into_type::<dom_objects::Element>() {
                write!(formatter, "<{}>", element.borrow().local_name())?;
            } else {
                write!(formatter, "NODE")?;
            }
            writeln!(formatter)?;

            let borrowed_node = node.borrow();
            if !borrowed_node.children().is_empty() {
                formatter.increase_indent();
                for child in borrowed_node.children() {
                    formatter.indent()?;
                    child.tree_fmt(formatter)?;
                }
                formatter.decrease_indent();
            }
        }
        Ok(())
    }
}
