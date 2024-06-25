//! A mark-and-sweep garbage collection

#![feature(must_not_suspend)]

mod cell;
mod heap;
mod node;
mod trace;

pub use cell::{GcCell, Ref};
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
pub struct Gc<T>
where
    T: 'static + ?Sized + Trace,
{
    is_rooted: Cell<bool>,

    /// The value stored
    ///
    /// It is a invariant of this type that `referenced_node` must always point to a valid
    /// [HeapNode<T>] (ie. it may never be dangling).
    referenced_node: Cell<NonNull<HeapNode<T>>>,
}

impl<T: Trace> Clone for Gc<T> {
    fn clone(&self) -> Self {
        Self::node_mut(self).increment_root_count();

        Self {
            // The new gc starts out on the stack
            is_rooted: Cell::new(true),

            referenced_node: self.referenced_node.clone(),
        }
    }
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

        // Since the value was just moved to the heap it no longer
        // needs to be rooted
        //
        // Note that we can't do this before moving it into the heapnode,
        // since allocating the node might trigger a garbage collection.
        unsafe { node.as_ref() }.value().unroot();

        Self {
            is_rooted: Cell::new(true),
            referenced_node: Cell::new(node),
        }
    }
}

unsafe impl<T> Trace for Gc<T>
where
    T: 'static + ?Sized + Trace,
{
    fn trace(&self) {
        Self::node(self).mark();
    }

    fn root(&self) {
        Self::node(self).value().root();
        Self::mark_as_root(self);
    }

    fn unroot(&self) {
        // SAFETY: Trace::unroot is only called when the value
        // is no longer accessible from the stack
        unsafe {
            Self::mark_as_no_root(self);
        }
    }
}

impl<T> Gc<T>
where
    T: 'static + ?Sized + Trace,
{
    /// Mark this GC as "accessible from the stack"
    ///
    /// As long as this value is not dropped or unrooted, the underlying
    /// gc node will not be deallocated.
    pub fn mark_as_root(value: &Self) {
        debug_assert!(!value.is_rooted.get());

        Self::node_mut(value).increment_root_count();
        value.is_rooted.set(true);
    }

    /// Mark this value as not on the stack
    ///
    /// # Safety
    /// Calling this might cause the value to be dropped if no other
    /// live references are around.
    pub unsafe fn mark_as_no_root(value: &Self) {
        debug_assert!(value.is_rooted.get());

        Self::node_mut(value).decrement_root_count();
        value.is_rooted.set(false);
    }

    #[must_use]
    pub fn node(value: &Self) -> &HeapNode<T> {
        let raw_ptr = value.referenced_node.get();

        // SAFETY: &self is a live reference, so the heap node is guaranteed to be alive
        unsafe { raw_ptr.as_ref() }
    }

    #[allow(clippy::mut_from_ref)]
    #[must_use]
    pub fn node_mut(value: &Self) -> &mut HeapNode<T> {
        let mut heap_node = value.referenced_node.get();

        // SAFETY: &self is a live reference, so the heap node is guaranteed to be alive
        unsafe { heap_node.as_mut() }
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
    T: 'static + ?Sized + Trace,
{
    fn drop(&mut self) {
        if self.is_rooted.get() {
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
