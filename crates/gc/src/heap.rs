use std::{
    cell::{Cell, RefCell},
    mem,
    ptr::{self, NonNull},
};

use crate::{node::HeapNode, Trace};

const COLLECT_IF_MEMORY_USAGE_ABOVE: usize = 0x1000;

thread_local! {
    pub static HEAP: RefCell<Heap> = RefCell::new(Heap {
        bytes_allocated: 0,
        collect_if_memory_usage_above: COLLECT_IF_MEMORY_USAGE_ABOVE,
        head: None,
    });
}

/// Forces a garbage collection
///
/// Returns the number of bytes that were freed
pub fn collect_garbage() -> usize {
    HEAP.with(|heap| heap.borrow_mut().collect_garbage())
}

pub(crate) struct Heap {
    bytes_allocated: usize,
    collect_if_memory_usage_above: usize,

    /// The most recently allocated gc node
    head: Option<NonNull<HeapNode<dyn Trace>>>,
}

impl Heap {
    pub(crate) unsafe fn register_node(&mut self, node: NonNull<HeapNode<dyn Trace>>) {
        debug_assert!(node.as_ref().next.get().is_none());

        // Make the new cell the head in the linked list of allocated nodes
        let old_head = self.head.replace(node);
        node.as_ref().next.set(old_head);

        self.bytes_allocated += mem::size_of_val(&node);

        if self.bytes_allocated > self.collect_if_memory_usage_above {
            self.collect_garbage();
        }
    }

    /// Performs a garbage collection on this threads heap
    ///
    /// Returns the number of bytes that were freed
    fn collect_garbage(&mut self) -> usize {
        log::debug!("Collecting garbage...");

        // Mark phase
        let mut next = self.head;
        while let Some(next_cell) = next {
            // SAFETY: All the pointers in the chain are guaranteed to point to
            //         valid HeapNodes
            let node = unsafe { next_cell.as_ref() };

            if node.num_roots() > 0 {
                node.mark();
            }
            next = node.next.get();
        }

        // Collect all unmarked nodes
        struct UnmarkedNode<'a> {
            node: NonNull<HeapNode<dyn Trace>>,
            linked_by: &'a Cell<Option<NonNull<HeapNode<dyn Trace>>>>,
        }

        let mut unmarked_nodes = vec![];
        let mut next = Cell::from_mut(&mut self.head);
        while let Some(next_node) = next.get() {
            // SAFETY: All the pointers in the chain are guaranteed to point to
            //         valid HeapNodes
            let node = unsafe { next_node.as_ref() };

            if node.is_marked() {
                node.unmark();
            } else {
                let unmarked_cell = UnmarkedNode {
                    node: next_node,
                    linked_by: next,
                };
                unmarked_nodes.push(unmarked_cell);
            }
            next = &node.next;
        }

        // Sweep Phase
        let mut total_freed_size = 0;
        for mut unmarked_node in unmarked_nodes {
            total_freed_size += mem::size_of_val(&unmarked_node.node);

            // Remove the unmarked node from the linked list
            // SAFETY: The node ptr is guaranteed to point to a valid node
            let node_to_be_dropped = unsafe { unmarked_node.node.as_mut() };
            unmarked_node.linked_by.set(node_to_be_dropped.next.get());

            // SAFETY: The node ptr is guaranteed to point to a valid node
            // The constructed box is immediately dropped
            let _ = unsafe { Box::from_raw(ptr::from_mut(node_to_be_dropped)) };
        }

        self.bytes_allocated -= total_freed_size;
        log::debug!("Freed 0x{total_freed_size:x} bytes during garbage collection");

        total_freed_size
    }
}

impl Drop for Heap {
    fn drop(&mut self) {
        self.collect_garbage();

        // Remaining memory is leaked
    }
}
