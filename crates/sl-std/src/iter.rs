//! Various extensions to [std::iter]

use std::iter::FusedIterator;

pub trait IteratorExtensions: Iterator {
    /// Creates an iterator that yields elements based on a predicate.
    fn take_while_including<P>(self, predicate: P) -> TakeWhileIncluding<Self, P>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool,
    {
        TakeWhileIncluding {
            iter: self,
            is_done: false,
            predicate,
        }
    }
}

impl<I: Iterator> IteratorExtensions for I {}

#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Clone)]
pub struct TakeWhileIncluding<I, P> {
    iter: I,
    is_done: bool,
    predicate: P,
}

impl<I: Iterator, P> Iterator for TakeWhileIncluding<I, P>
where
    P: FnMut(&I::Item) -> bool,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_done {
            return None;
        }

        let element = self.iter.next()?;
        if !(self.predicate)(&element) {
            // Stop iterating, but return the last item
            self.is_done = true;
        }

        Some(element)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.is_done {
            (0, Some(0))
        } else {
            (0, self.iter.size_hint().1)
        }
    }
}

impl<I, P> FusedIterator for TakeWhileIncluding<I, P>
where
    I: Iterator + FusedIterator,
    P: FnMut(&I::Item) -> bool,
{
}

/// Like [std::slice::Split] except it allows to split on a sequence
/// of elements instead of just a single one.
#[derive(Clone, Copy)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct MultiElementSplit<'a, T, P, const N: usize>
where
    P: FnMut(&[T; N]) -> bool,
{
    elements: &'a [T],
    predicate: P,
    is_finished: bool,
}

impl<'a, T, P, const N: usize> MultiElementSplit<'a, T, P, N>
where
    P: FnMut(&[T; N]) -> bool,
{
    #[inline]
    pub const fn new(elements: &'a [T], predicate: P) -> Self {
        Self {
            elements,
            predicate,
            is_finished: false,
        }
    }

    #[inline]
    #[must_use]
    pub const fn as_slice(&self) -> &[T] {
        self.elements
    }

    #[inline]
    pub fn finish(&mut self) {
        self.is_finished = true;
    }
}

impl<'a, T, P, const N: usize> Iterator for MultiElementSplit<'a, T, P, N>
where
    P: FnMut(&[T; N]) -> bool,
{
    type Item = &'a [T];

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_finished {
            return None;
        }

        let index_of_next_match = self
            .elements
            .array_windows()
            .position(|w| (self.predicate)(w));
        match index_of_next_match {
            None => {
                self.finish();
                Some(self.elements)
            },
            Some(index) => {
                let to_return = &self.elements[..index];
                self.elements = &self.elements[index + N..];
                Some(to_return)
            },
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.elements.is_empty() {
            (0, Some(0))
        } else {
            let max_future_items = self.elements.len() / N + 1;
            (1, Some(max_future_items))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn take_until() {
        let mut iter = (1..5).take_while_including(|&i| i < 3);
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn multi_element_split() {
        let mut basic = MultiElementSplit::new(b"Dora the Explorer", |w| w == b"or");
        assert_eq!(basic.next(), Some(b"D".as_slice()));
        assert_eq!(basic.next(), Some(b"a the Expl".as_slice()));
        assert_eq!(basic.next(), Some(b"er".as_slice()));
        assert!(basic.next().is_none());

        let mut empty_items = MultiElementSplit::new(b"abab", |w| w == b"ab");
        assert_eq!(empty_items.next(), Some(b"".as_slice()));
        assert_eq!(empty_items.next(), Some(b"".as_slice()));
        assert_eq!(empty_items.next(), Some(b"".as_slice()));
        assert!(empty_items.next().is_none());

        let mut overlapping_items = MultiElementSplit::new(b"ababab", |w| w == b"abab");
        assert_eq!(overlapping_items.next(), Some(b"".as_slice()));
        assert_eq!(overlapping_items.next(), Some(b"ab".as_slice()));
        assert!(overlapping_items.next().is_none());
    }
}
