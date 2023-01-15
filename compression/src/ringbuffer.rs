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
    /// last pushed. However, you may not retrieve more than `size` previous elements.
    ///
    /// # Panics
    /// This function panics if `index >=  buffer size`
    pub fn nth_last(&self, index: usize) -> &T {
        assert!(index < self.size());
        &self.elements[(self.ptr + self.size() - index - 1) % self.size()]
        // let unwrapped_index = (index + 1) % self.size();

        // if self.ptr < unwrapped_index {
        //     // wrap back around
        //     &self.elements[self.ptr + self.size() - unwrapped_index]
        // } else {
        //     &self.elements[self.ptr - unwrapped_index]
        // }
    }
}

#[cfg(test)]
mod tests {
    use super::RingBuffer;

    #[test]
    fn test_ringbuffer() {
        let mut buffer = RingBuffer::new(vec![3, 2, 1]);

        assert_eq!(*buffer.nth_last(0), 1);
        assert_eq!(*buffer.nth_last(1), 2);
        assert_eq!(*buffer.nth_last(2), 3);

        buffer.push(4);
        // Internal buffer should now look like this:
        // [4, 2, 1]
        //     ^_ self.ptr

        assert_eq!(*buffer.nth_last(0), 4);
        assert_eq!(*buffer.nth_last(1), 1);
        assert_eq!(*buffer.nth_last(2), 2);

        buffer.push(5);
        // Internal buffer should now look like this:
        // [5, 4, 1]
        //        ^_ self.ptr

        assert_eq!(*buffer.nth_last(0), 5);
        assert_eq!(*buffer.nth_last(1), 4);
        assert_eq!(*buffer.nth_last(2), 1);

        buffer.push(6);
        // Internal buffer should now look like this:
        // [5, 4, 6]
        //  ^_ self.ptr

        assert_eq!(*buffer.nth_last(0), 6);
        assert_eq!(*buffer.nth_last(1), 5);
        assert_eq!(*buffer.nth_last(2), 3);
    }
}
