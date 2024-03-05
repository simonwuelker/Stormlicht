use std::{cell::Cell, ptr::NonNull};

use crate::{heap::HEAP, Trace};

const MARKED_BIT: usize = 1 << (usize::BITS - 1);
const ROOTS_MASK: usize = !MARKED_BIT;

pub(crate) struct HeapNode<T: ?Sized> {
    /// Contains root count and whether or not the node is marked
    ///
    /// Highest bit indicates mark state, lower bits are the root count.
    pub(crate) flags: Cell<usize>,

    /// [HeapNodes](HeapNode) make up a linked list, to keep track of all allocated objects
    pub(crate) next: Cell<Option<NonNull<HeapNode<dyn Trace>>>>,

    /// The actual value allocated
    pub(crate) value: T,
}

impl<T> HeapNode<T>
where
    T: Trace + 'static,
{
    pub fn new(value: T) -> NonNull<Self> {
        let node = Self {
            flags: Cell::new(0x1), // Not marked, one root
            next: Cell::new(None),
            value,
        };

        let node = NonNull::from(Box::leak(Box::new(node)));

        // SAFETY: The ptr is valid, as we just constructed it
        unsafe { HEAP.with(|heap| heap.borrow_mut().register_node(node)) }

        node
    }
}

impl<T> HeapNode<T>
where
    T: ?Sized + Trace,
{
    #[inline]
    pub const fn value(&self) -> &T {
        &self.value
    }

    /// Marks the cell and all its successors
    pub fn mark(&self) {
        if !self.is_marked() {
            self.flags.set(self.flags.get() | MARKED_BIT);

            // Also mark all of the connected cells
            self.value.trace();
        }
    }

    pub fn unmark(&self) {
        self.flags.set(self.flags.get() & ROOTS_MASK);
    }

    #[must_use]
    pub fn num_roots(&self) -> usize {
        self.flags.get() & ROOTS_MASK
    }

    #[must_use]
    pub fn is_marked(&self) -> bool {
        self.flags.get() & MARKED_BIT != 0
    }

    pub fn decrement_root_count(&mut self) {
        self.flags.set(self.flags.get() - 1);
    }

    pub fn increment_root_count(&mut self) {
        if self.num_roots() == ROOTS_MASK {
            panic!("Maximum number of gc roots exceeded");
        }

        self.flags.set(self.flags.get() + 1);
    }
}
