//! Implements a [circular buffer](https://en.wikipedia.org/wiki/Circular_buffer) which can hold a fixed number of items.

#[derive(Debug)]
pub struct RingBuffer<T> {
    elements: Vec<T>,
    ptr: usize,
}

impl<T> RingBuffer<T> {
    pub fn new(elements: Vec<T>) -> Self {
        Self {
            elements: elements,
            ptr: 0,
        }
    }

    pub fn size(&self) -> usize {
        self.elements.len()
    }

    pub fn push(&mut self, element: T) {
        self.elements[self.ptr] = element;
        self.ptr += 1;
        self.ptr %= self.size();
    }

    /// Get the nth previous element from the ringbuffer.
    ///
    /// Note that the index is 0-based, so `nth_last(0)` returns the element that was
    /// last pushed.
    pub fn nth_last(&self, index: usize) -> &T {
        let unwrapped_index = (index + 1) % self.size();

        if self.ptr < unwrapped_index {
            // wrap back around
            &self.elements[self.ptr + self.size() - unwrapped_index]
        } else {
            &self.elements[self.ptr - unwrapped_index]
        }
    }
}
