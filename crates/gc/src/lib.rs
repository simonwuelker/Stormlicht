//! A mark-and-sweep garbage collection

mod heap;
mod node;
mod trace;

pub use heap::collect_garbage;
use node::HeapNode;
pub use trace::Trace;

use std::{cell::Cell, fmt, ops::Deref, ptr::NonNull};

/// A pointer to the gc heap
///
/// Methods are implemented on the `Gc` itself, so as to not
/// interfer with the methods on `T`.
///
/// Cloning a [Gc] does not perform a deep copy.
#[derive(Clone)]
pub struct Gc<T>
where
    T: 'static + Trace,
{
    is_rooted: bool,

    /// The value stored
    ///
    /// It is a invariant of this type that `referenced_node` must always point to a valid
    /// [HeapNode<T>] (ie. it may never be dangling).
    referenced_node: Cell<NonNull<HeapNode<T>>>,
}

impl<T> Gc<T>
where
    T: 'static + Trace,
{
    /// Allocate a value on the gc-heap
    ///
    /// The new pointer starts out rooted.
    #[must_use]
    pub fn new(value: T) -> Self {
        // Allocate a new node on the thread-local heap for this value
        let node = HeapNode::new(value);

        let gc = Self {
            is_rooted: true,
            referenced_node: Cell::new(node),
        };

        gc
    }

    fn make_root(value: &mut Self) {
        debug_assert!(!value.is_rooted);

        Self::node_mut(value).increment_root_count();
        value.is_rooted = true;
    }

    fn unroot(value: &mut Self) {
        debug_assert!(value.is_rooted);

        Self::node_mut(value).decrement_root_count();
        value.is_rooted = false;
    }

    fn node(value: &Self) -> &HeapNode<T> {
        let raw_ptr = value.referenced_node.get();

        // SAFETY: self.cell must always point to a GcCell
        unsafe { raw_ptr.as_ref() }
    }

    fn node_mut(value: &mut Self) -> &mut HeapNode<T> {
        let raw_ptr = value.referenced_node.get_mut();

        // SAFETY: self.cell must always point to a GcCell
        unsafe { raw_ptr.as_mut() }
    }

    pub fn mark(value: &Self) {
        Self::node(value).mark()
    }
}

impl<T> fmt::Debug for Gc<T>
where
    T: 'static + Trace + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Self::node(self).value().fmt(f)
    }
}

impl<T> fmt::Display for Gc<T>
where
    T: 'static + Trace + fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Self::node(self).value().fmt(f)
    }
}

impl<T> Drop for Gc<T>
where
    T: 'static + Trace,
{
    fn drop(&mut self) {
        if self.is_rooted {
            Self::node_mut(self).decrement_root_count();
        }
    }
}

impl<T> Deref for Gc<T>
where
    T: 'static + Trace,
{
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        Gc::node(self).value()
    }
}
