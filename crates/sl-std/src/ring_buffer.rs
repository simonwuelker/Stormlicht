//! Implements a [circular buffer](https://en.wikipedia.org/wiki/Circular_buffer) which can hold a fixed number of items.

use std::mem;

/// A circular buffer capable of storing up to `N` items at once
#[derive(Debug)]
pub struct RingBuffer<T, const N: usize> {
    elements: [mem::MaybeUninit<T>; N],
    write_head: usize,
    read_head: usize,

    /// Distinguishes between the "full" and the "empty" state
    ///
    /// Both of these cases look the same from the outside, as
    /// in both cases the `write_head` is the same as the `read_head`.
    is_full: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct PushError;

impl<T, const N: usize> Default for RingBuffer<T, N> {
    fn default() -> Self {
        Self {
            elements: mem::MaybeUninit::uninit_array(),
            write_head: 0,
            read_head: 0,
            is_full: false,
        }
    }
}

impl<T, const N: usize> RingBuffer<T, N> {
    #[must_use]
    #[inline]
    pub const fn max_size(&self) -> usize {
        N
    }

    #[inline]
    #[must_use]
    pub const fn is_full(&self) -> bool {
        self.read_head == self.write_head && self.is_full
    }

    /// Return `true` if there are no elements in the buffer
    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.read_head == self.write_head && !self.is_full
    }

    #[must_use]
    pub fn pop_front(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        let item = self
            .elements
            .get(self.read_head)
            .expect("Read head outside of elements array");

        self.read_head = self.read_head.wrapping_add(1) % self.max_size();

        // SAFETY:
        // * The value is initialized, since write_head has moved past it
        // * The value will never be read again, since we incremented the read head past it
        let item = unsafe { item.assume_init_read() };

        // If the buffer was full right before the pop operation, then it is now no longer full
        self.is_full = false;

        Some(item)
    }

    /// Push an element to the buffer
    ///
    /// # Panics
    /// This function panics if there is no more space available.
    pub fn push(&mut self, element: T) {
        if let Err(error) = self.try_push(element) {
            panic!("Failed to push element; buffer is full: {error:?}");
        }
    }

    /// Push an element to the buffer, possibly overwriting the oldest existing element
    pub fn push_overwriting(&mut self, element: T) {
        // Make space for the element if necessary
        if self.is_full() {
            // Note that we can't simply overwrite the element, as we might need
            // to run the drop implementation for the element
            let _ = self.pop_front();
        }

        self.push(element);
    }

    /// Try to push an element to the buffer
    ///
    /// If the buffer is already full, an error is returned.
    pub fn try_push(&mut self, element: T) -> Result<(), PushError> {
        if self.is_full() {
            return Err(PushError);
        }

        self.elements[self.write_head].write(element);
        self.write_head = self.write_head.wrapping_add(1) % self.max_size();

        if self.write_head == self.read_head {
            // If the heads are equal after a push operation, then the buffer is full
            self.is_full = true;
        }

        Ok(())
    }

    /// Get the nth previous element from the ringbuffer.
    ///
    /// Note that the index is 0-based, so `nth_last(0)` returns the element that was
    /// last pushed. However, you may not retrieve more than `size` previous elements.
    ///
    /// # Panics
    /// This function panics if `index >=  buffer size`
    #[inline]
    #[must_use]
    pub fn nth_last(&self, index: usize) -> &T {
        assert!(index < self.max_size());
        let index = (self.write_head + self.max_size() - index - 1) % self.max_size();

        // SAFETY: elements[index] is initialized, since it is between the read head and the write head
        let element = unsafe { self.elements[index].assume_init_ref() };

        element
    }
}

impl<T, const N: usize> Drop for RingBuffer<T, N> {
    fn drop(&mut self) {
        // We store the elements in the buffer as MaybeUninits, so their drop code won't be
        // run automatically
        while let Some(element) = self.pop_front() {
            drop(element);
        }
    }
}

impl<T, const N: usize> From<[T; N]> for RingBuffer<T, N> {
    fn from(value: [T; N]) -> Self {
        let mut buffer = Self::default();
        for element in value {
            buffer.push(element);
        }
        buffer
    }
}

#[cfg(test)]
mod tests {
    use super::RingBuffer;

    #[test]
    fn fill_buffer() {
        let mut buffer: RingBuffer<i32, 3> = RingBuffer::default();

        // Fill the buffer
        buffer.push(1);
        buffer.push(2);
        buffer.push(3);

        // Further push operations will fail
        assert!(buffer.try_push(4).is_err());
    }

    #[test]
    fn test_ringbuffer() {
        let mut buffer = RingBuffer::from([3, 2, 1]);

        assert_eq!(*buffer.nth_last(0), 1);
        assert_eq!(*buffer.nth_last(1), 2);
        assert_eq!(*buffer.nth_last(2), 3);

        buffer.push_overwriting(4);
        // Internal buffer should now look like this:
        // [4, 2, 1]
        //     ^_ self.ptr

        assert_eq!(*buffer.nth_last(0), 4);
        assert_eq!(*buffer.nth_last(1), 1);
        assert_eq!(*buffer.nth_last(2), 2);

        buffer.push_overwriting(5);
        // Internal buffer should now look like this:
        // [5, 4, 1]
        //        ^_ self.ptr

        assert_eq!(*buffer.nth_last(0), 5);
        assert_eq!(*buffer.nth_last(1), 4);
        assert_eq!(*buffer.nth_last(2), 1);

        buffer.push_overwriting(6);
        // Internal buffer should now look like this:
        // [5, 4, 6]
        //  ^_ self.ptr

        assert_eq!(*buffer.nth_last(0), 6);
        assert_eq!(*buffer.nth_last(1), 5);
        assert_eq!(*buffer.nth_last(2), 4);
    }
}
