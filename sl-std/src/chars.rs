#[derive(Clone, Copy, Debug)]
enum State {
    BeforeStart(usize),
    Within,
    AfterEnd(usize),
}

#[derive(Clone, Copy, Debug)]
pub struct ReversibleCharIterator<'str> {
    source: &'str str,
    // The current byte position of the iterator
    pos: usize,
    state: State,
}

#[derive(Debug)]
pub struct ForwardCharIterator<'iter, 'str> {
    inner: &'iter mut ReversibleCharIterator<'str>,
}

#[derive(Debug)]
pub struct BackwardCharIterator<'iter, 'str> {
    inner: &'iter mut ReversibleCharIterator<'str>,
}

impl<'str> ReversibleCharIterator<'str> {
    pub fn new(source: &'str str) -> Self {
        Self {
            source,
            pos: 0,
            state: State::Within,
        }
    }

    pub fn forward<'iter>(&'iter mut self) -> ForwardCharIterator<'iter, 'str> {
        ForwardCharIterator { inner: self }
    }

    pub fn backward<'iter>(&'iter mut self) -> BackwardCharIterator<'iter, 'str> {
        BackwardCharIterator { inner: self }
    }
}

impl<'iter, 'str> ForwardCharIterator<'iter, 'str> {
    pub fn finish(self) {}
}

impl<'iter, 'str> BackwardCharIterator<'iter, 'str> {
    pub fn finish(self) {}
}

impl<'iter, 'str> Iterator for ForwardCharIterator<'iter, 'str> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        match self.inner.state {
            State::BeforeStart(ref mut n) => {
                *n -= 1;

                if *n == 0 {
                    self.inner.state = State::Within;
                }

                None
            },
            State::Within => {
                debug_assert!(self.inner.source.is_char_boundary(self.inner.pos));

                let c = self.inner.source[self.inner.pos..]
                    .chars()
                    .nth(0)
                    .expect("inner.pos was a char boundary");
                self.inner.pos += c.len_utf8();

                if self.inner.pos == self.inner.source.len() {
                    self.inner.state = State::AfterEnd(0)
                }

                Some(c)
            },
            State::AfterEnd(ref mut n) => {
                *n += 1;
                None
            },
        }
    }
}

impl<'iter, 'str> Iterator for BackwardCharIterator<'iter, 'str> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        match self.inner.state {
            State::BeforeStart(ref mut n) => {
                *n += 1;

                None
            },
            State::Within => {
                debug_assert!(self.inner.source.is_char_boundary(self.inner.pos));

                // Find the byte position of the previous character
                self.inner.pos = self.inner.source.floor_char_boundary(self.inner.pos - 1);
                let c = self.inner.source[self.inner.pos..]
                    .chars()
                    .nth(0)
                    .expect("inner.pos was a char boundary");

                if self.inner.pos == 0 {
                    self.inner.state = State::BeforeStart(0)
                }

                Some(c)
            },
            State::AfterEnd(ref mut n) => {
                *n -= 1;

                if *n == 0 {
                    self.inner.state = State::Within;
                }

                None
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ReversibleCharIterator;

    #[test]
    fn forward_backward() {
        let mut iter = ReversibleCharIterator::new("💚💙💜");
        let mut forward = iter.forward();

        // Forward pass, expect all characters in order
        assert_eq!(forward.next(), Some('💚'));
        assert_eq!(forward.next(), Some('💙'));
        assert_eq!(forward.next(), Some('💜'));

        // Consume one character past the end
        assert_eq!(forward.next(), None);
        forward.finish();

        let mut backward = iter.backward();

        // Return to the end of the string
        assert_eq!(backward.next(), None);

        // Backwards pass, expect all characters in reverse order
        assert_eq!(backward.next(), Some('💜'));
        assert_eq!(backward.next(), Some('💙'));
        assert_eq!(backward.next(), Some('💚'));

        assert_eq!(backward.next(), None);
        backward.finish();

        // Test going back and forth in the middle
        let mut forward = iter.forward();
        assert_eq!(forward.next(), None);

        // Forward pass, expect all characters in order
        assert_eq!(forward.next(), Some('💚'));
        assert_eq!(forward.next(), Some('💙'));
        forward.finish();

        assert_eq!(iter.backward().next(), Some('💙'));
        assert_eq!(iter.backward().next(), Some('💚'));
    }
}
