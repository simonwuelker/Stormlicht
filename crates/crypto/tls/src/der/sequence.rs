use super::{Error, Item};

use std::iter;

#[derive(Clone, Copy, Debug)]
pub struct Sequence<'a> {
    bytes: &'a [u8],
}

impl<'a> Sequence<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }
}

impl<'a> iter::Iterator for Sequence<'a> {
    type Item = Result<Item<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bytes.is_empty() {
            return None;
        }

        match Item::parse(self.bytes) {
            Ok((item, length)) => {
                self.bytes = &self.bytes[length..];
                Some(Ok(item))
            },
            Err(e) => Some(Err(e)),
        }
    }
}

impl<'a> iter::FusedIterator for Sequence<'a> {}
