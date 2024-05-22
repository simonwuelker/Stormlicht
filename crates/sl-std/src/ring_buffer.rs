//! Implements a [circular buffer](https://en.wikipedia.org/wiki/Circular_buffer) which can hold a fixed number of items.

use std::{iter::FusedIterator, mem};

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
    // Return the number of elements currently stored in the buffer
    #[must_use]
    pub const fn len(&self) -> usize {
        if self.is_full() {
            return self.max_size();
        }

        if self.read_head <= self.write_head {
            self.write_head - self.read_head
        } else {
            N - self.read_head + self.write_head
        }
    }

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

    /// Peek `n` elements ahead of the current one
    #[must_use]
    pub fn peek_front(&self, n: usize) -> Option<&T> {
        if self.len() <= n {
            return None;
        }

        let index = self.read_head.wrapping_add(n) % self.max_size();

        // SAFETY:
        // Due to the length check above, we know that the element at the given index must be initialized
        let element = unsafe { self.elements[index].assume_init_ref() };

        Some(element)
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
    #[inline]
    #[must_use]
    pub fn peek_back(&self, index: usize) -> Option<&T> {
        if self.len() <= index {
            return None;
        }

        let index = (self.write_head + self.max_size() - index - 1) % self.max_size();

        // SAFETY:
        // We can be sure that elements[index] is initialized due to the range check above
        let element = unsafe { self.elements[index].assume_init_ref() };

        Some(element)
    }

    #[inline]
    #[must_use]
    pub const fn iter(&self) -> RingBufferIterator<'_, T, N> {
        RingBufferIterator {
            current_index: 0,
            ring_buffer: self,
        }
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

impl<T, const N: usize> Clone for RingBuffer<T, N>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        let mut elements = mem::MaybeUninit::uninit_array();

        for (i, element) in self.iter().enumerate() {
            elements[i].write(element.clone());
        }
        let write_head = self.len() % N;

        Self {
            elements,
            write_head,
            read_head: 0,
            is_full: self.is_full,
        }
    }
}

#[derive(Clone, Copy)]
pub struct RingBufferIterator<'a, T, const N: usize> {
    current_index: usize,
    ring_buffer: &'a RingBuffer<T, N>,
}

impl<'a, T, const N: usize> Iterator for RingBufferIterator<'a, T, N> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let element = self.ring_buffer.peek_front(self.current_index);
        self.current_index += 1;
        element
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let elements_left = self.ring_buffer.len() - self.current_index;
        (elements_left, Some(elements_left))
    }
}

impl<'a, T, const N: usize> FusedIterator for RingBufferIterator<'a, T, N> {}

impl<'a, T, const N: usize> ExactSizeIterator for RingBufferIterator<'a, T, N> {
    fn len(&self) -> usize {
        self.ring_buffer.len() - self.current_index
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use super::RingBuffer;

    /// Creates a empty ringbuffer whose read/write heads aren't aligned to
    /// the start element.
    /// This should help cover some edge cases.
    fn unaligned_ringbuf() -> RingBuffer<i32, 3> {
        let mut buffer: RingBuffer<i32, 3> = RingBuffer::default();

        buffer.push(1);
        let _ = buffer.pop_front();

        buffer
    }

    #[test]
    fn fill_buffer() {
        let mut buffer = unaligned_ringbuf();

        // Fill the buffer
        buffer.push(1);
        buffer.push(2);
        buffer.push(3);

        // Further push operations will fail
        assert!(buffer.try_push(4).is_err());
    }

    #[test]
    fn peek() {
        let mut buffer = unaligned_ringbuf();

        // Peeking on an empty buffer fails
        assert!(buffer.peek_front(0).is_none());

        buffer.push(1);

        assert_matches!(buffer.peek_front(0), Some(1));
        assert!(buffer.peek_front(1).is_none());

        buffer.push(2);
        buffer.push(3);

        // Test peeking multiple elements ahead
        assert_matches!(buffer.peek_front(1), Some(2));
        assert_matches!(buffer.peek_front(2), Some(3));

        // Peeking to an index larger than the end of the buffer always fails,
        // even if the buffer is completely full
        assert!(buffer.peek_front(4).is_none());
    }

    #[test]
    fn len() {
        let mut buffer = unaligned_ringbuf();

        assert_eq!(buffer.len(), 0);

        buffer.push(1);
        buffer.push(2);

        assert_eq!(buffer.len(), 2);

        _ = buffer.pop_front();

        assert_eq!(buffer.len(), 1);
    }

    #[test]
    fn peek_back() {
        let mut buffer = RingBuffer::from([3, 2, 1]);

        assert_matches!(buffer.peek_back(0), Some(1));
        assert_matches!(buffer.peek_back(1), Some(2));
        assert_matches!(buffer.peek_back(2), Some(3));

        buffer.push_overwriting(4);
        // Internal buffer should now look like this:
        // [4, 2, 1]
        //     ^_ self.ptr

        assert_matches!(buffer.peek_back(0), Some(4));
        assert_matches!(buffer.peek_back(1), Some(1));
        assert_matches!(buffer.peek_back(2), Some(2));

        buffer.push_overwriting(5);
        // Internal buffer should now look like this:
        // [5, 4, 1]
        //        ^_ self.ptr

        assert_matches!(buffer.peek_back(0), Some(5));
        assert_matches!(buffer.peek_back(1), Some(4));
        assert_matches!(buffer.peek_back(2), Some(1));

        buffer.push_overwriting(6);
        // Internal buffer should now look like this:
        // [5, 4, 6]
        //  ^_ self.ptr

        assert_matches!(buffer.peek_back(0), Some(6));
        assert_matches!(buffer.peek_back(1), Some(5));
        assert_matches!(buffer.peek_back(2), Some(4));

        // Peeking back outside of the buffer should always fail
        assert!(buffer.peek_back(3).is_none());
    }

    #[test]
    fn iter() {
        let mut buffer = unaligned_ringbuf();

        buffer.push(1);
        buffer.push(2);
        buffer.push(3);

        let mut items = buffer.iter();

        assert_eq!(items.len(), 3);

        assert_matches!(items.next(), Some(1));
        assert_matches!(items.next(), Some(2));
        assert_matches!(items.next(), Some(3));
        assert!(items.next().is_none());
    }

    #[test]
    fn clone() {
        let mut buffer = unaligned_ringbuf();

        buffer.push(1);
        buffer.push(2);
        buffer.push(3);

        let cloned_buffer = buffer.clone();

        assert_eq!(cloned_buffer.len(), buffer.len());

        for (a, b) in buffer.iter().zip(cloned_buffer.iter()) {
            assert_eq!(a, b);
        }
    }
}
