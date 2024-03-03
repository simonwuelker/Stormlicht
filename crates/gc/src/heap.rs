use std::{
    cell::{Cell, RefCell},
    mem,
    ptr::{self, NonNull},
};

use crate::{cell::GcCell, Trace};

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

    /// The most recently allocated gc cell
    head: Option<NonNull<GcCell<dyn Trace>>>,
}

impl Heap {
    pub(crate) unsafe fn register_cell(&mut self, cell: NonNull<GcCell<dyn Trace>>) {
        debug_assert!(cell.as_ref().next.get().is_none());

        // Make the new cell the head in the linked list of allocated cells
        let old_head = self.head.replace(cell);
        cell.as_ref().next.set(old_head);

        self.bytes_allocated += mem::size_of_val(&cell);

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
            //         valid GcCells
            let cell = unsafe { next_cell.as_ref() };

            if cell.num_roots() > 0 {
                cell.mark();
            }
            next = cell.next.get();
        }

        // Collect all unmarked nodes
        struct UnmarkedCell<'a> {
            cell: NonNull<GcCell<dyn Trace>>,
            linked_by: &'a Cell<Option<NonNull<GcCell<dyn Trace>>>>,
        }

        let mut unmarked_cells = vec![];
        let mut next = Cell::from_mut(&mut self.head);
        while let Some(next_cell) = next.get() {
            // SAFETY: All the pointers in the chain are guaranteed to point to
            //         valid GcCells
            let cell = unsafe { next_cell.as_ref() };

            if cell.is_marked() {
                cell.unmark();
            } else {
                let unmarked_cell = UnmarkedCell {
                    cell: next_cell,
                    linked_by: next,
                };
                unmarked_cells.push(unmarked_cell);
            }
            next = &cell.next;
        }

        // Sweep Phase
        let mut total_freed_size = 0;
        for mut unmarked_cell in unmarked_cells {
            total_freed_size += mem::size_of_val(&unmarked_cell.cell);

            // Remove the unmarked cell from the linked list
            // SAFETY: The cell ptr is guaranteed to point to a valid cell
            let cell_to_be_dropped = unsafe { unmarked_cell.cell.as_mut() };
            unmarked_cell.linked_by.set(cell_to_be_dropped.next.get());

            // SAFETY: The cell ptr is guaranteed to point to a valid cell
            unsafe { ptr::drop_in_place(ptr::from_mut(cell_to_be_dropped)) };
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
