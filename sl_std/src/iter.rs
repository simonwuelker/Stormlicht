//! Various extensions to [std::iter]

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
    use super::MultiElementSplit;

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
