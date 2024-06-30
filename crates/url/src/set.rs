//! Utilities for defining a subset of ASCII characters

use std::mem;

use sl_std::ascii;

type Block = usize;

const ASCII_MAX: u8 = 0x80;
const BITS_PER_BLOCK: usize = mem::size_of::<usize>() * 8;
const NUM_BLOCKS: usize = ASCII_MAX as usize / BITS_PER_BLOCK;

#[derive(Clone, Copy, Default)]
pub struct AsciiSet {
    // Relies on the fact that ASCII_MAX is a multiple of the pointer width
    bits: [Block; NUM_BLOCKS],
}

impl AsciiSet {
    pub const EMPTY: Self = Self {
        bits: [0; NUM_BLOCKS],
    };

    #[must_use]
    pub const fn from_range(start: u8, end: u8) -> Self {
        let mut set = Self::EMPTY;

        let mut i = start;
        while i < end {
            let c = ascii::Char::from_u8(i).unwrap();
            set = set.add(c);
            i += 1;
        }

        set
    }

    #[must_use]
    pub const fn merge(self, other: Self) -> Self {
        let mut result = Self::EMPTY;

        let mut i = 0;
        while i < NUM_BLOCKS {
            result.bits[i] = self.bits[i] | other.bits[i];
            i += 1;
        }

        result
    }

    /// Test whether or not the set contains the given character
    #[inline]
    #[must_use]
    pub const fn contains(&self, c: ascii::Char) -> bool {
        let index = (c as usize) / BITS_PER_BLOCK;
        let offset = (c as usize) % BITS_PER_BLOCK;
        self.bits[index] & (1 << offset) != 0
    }

    #[must_use]
    pub const fn add(mut self, c: ascii::Char) -> Self {
        let index = (c as usize) / BITS_PER_BLOCK;
        let offset = (c as usize) % BITS_PER_BLOCK;
        self.bits[index] |= 1 << offset;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_set_is_empty() {
        let set = AsciiSet::default();

        for i in 0..ASCII_MAX {
            let c = ascii::Char::from_u8(i).unwrap();
            assert!(!set.contains(c));
        }
    }

    #[test]
    fn add_contains() {
        let mut set = AsciiSet::default();

        const SET_START: u8 = b'a';
        const SET_END: u8 = b'z';

        for i in SET_START..=SET_END {
            let c = ascii::Char::from_u8(i).unwrap();
            set = set.add(c);
        }

        for i in 0..ASCII_MAX {
            let c = ascii::Char::from_u8(i).unwrap();

            if (SET_START..=SET_END).contains(&i) {
                assert!(set.contains(c));
            } else {
                assert!(!set.contains(c));
            }
        }
    }
}
