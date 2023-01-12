//! Huffman Tree implementation.
//!
//! For the purposes of this module, "Symbol" shall refer to an unencoded
//! codepoint and "Code" shall refer to an encoded codepoint.

use std::fmt;

use crate::compress::bit_reader::BitReader;

/// Tuple of (data, nbits) for representing an arbitrary number of bits
#[derive(Clone, Copy)]
pub struct Bits<T: Copy>(T, usize);
pub type Code = Bits<usize>;

#[derive(Debug)]
pub struct HuffmanTree<T: PartialOrd + PartialEq> {
    /// A value of `Some(_)` means that the node is a leaf node and there is a symbol
    /// associated with the Code.
    /// A value of `None` means that the node is not a leaf node.
    nodes: Vec<Option<T>>,
}

impl<T: PartialOrd + PartialEq + Clone> HuffmanTree<T> {
    pub fn new_infer_codes(symbols: &[T], lengths: &[usize]) -> Self {
        assert_eq!(
            symbols.len(),
            lengths.len(),
            "Every symbol must be assigned exactly one length"
        );

        let max_bits = *lengths.iter().max().unwrap_or(&0);
        let mut length_count = vec![0_usize; max_bits + 1];

        for length in lengths.iter() {
            length_count[*length] += 1;
        }

        let mut next_code = Vec::with_capacity(max_bits);
        let mut code = 0;
        length_count[0] = 0;

        for bits in 1..=max_bits {
            code = (code + length_count[bits - 1]) << 1;
            next_code.push(code);
        }

        let mut tree = Self::new_with_depth(max_bits);

        // The alphabet is assumed to be sorted by the caller
        for (symbol, length) in symbols.iter().zip(lengths) {
            if *length != 0 {
                let code = Code::new(next_code[length - 1], *length);
                tree.insert(code, symbol.clone());

                next_code[length - 1] += 1;
            }
        }

        tree
    }

    pub fn new_with_depth(depth: usize) -> Self {
        Self {
            nodes: vec![None; (1 << (depth + 1)) - 1],
        }
    }

    fn insert(&mut self, at: Code, symbol: T) {
        let insert_index = (1 << at.len()) - 1 + at.val() as usize;

        debug_assert!(self.nodes[insert_index].is_none());

        self.nodes[insert_index] = Some(symbol);
    }

    pub fn lookup_incrementally(&self, reader: &mut BitReader) -> Result<Option<&T>, ()> {
        let mut val = reader.read_bits::<usize>(1).map_err(|_| ())?;

        for length in 1..usize::BITS as usize {
            let code = Code::new(val, length);

            if let Some(symbol) = self.lookup_symbol(code) {
                return Ok(Some(symbol));
            }

            val <<= 1;
            val |= reader.read_bits::<usize>(1).map_err(|_| ())?;
        }

        Ok(None)
    }

    /// Lookup the code for a specific symbol
    pub fn lookup_symbol(&self, at: Code) -> &Option<T> {
        let insert_index = (1 << at.len()) - 1 + at.val() as usize;

        if insert_index > self.nodes.len() {
            &None
        } else {
            &self.nodes[insert_index]
        }
    }
}

impl<T: Copy> Bits<T> {
    pub fn new(bits: T, num_bits: usize) -> Self {
        Self(bits, num_bits)
    }

    pub fn val(&self) -> T {
        self.0
    }

    pub fn len(&self) -> usize {
        self.1
    }
}

impl<T: Copy + fmt::Display + fmt::Binary + PartialEq> fmt::Debug for Bits<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let unpadded = format!("{:b}", self.val());
        write!(
            f,
            "{}{}",
            "0".repeat(self.1 as usize - unpadded.len()),
            self.0
        )
    }
}

impl<T: Copy + PartialEq> PartialEq for Bits<T> {
    fn eq(&self, other: &Self) -> bool {
        self.val() == other.val() && self.len() == other.len()
    }
}

impl<T: Copy + PartialOrd> PartialOrd for Bits<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.len() != other.len() {
            None
        } else {
            self.val().partial_cmp(&other.val())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_codes_by_length() {
        // Example 1 Section 3.4
        let symbols = vec!['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H'];
        let lengths = vec![3, 3, 3, 3, 3, 2, 4, 4];
        let htree = HuffmanTree::new_infer_codes(&symbols, &lengths);

        assert_eq!(*htree.lookup_symbol(Code::new(0b010, 3)), Some('A'));
        assert_eq!(*htree.lookup_symbol(Code::new(0b011, 3)), Some('B'));
        assert_eq!(*htree.lookup_symbol(Code::new(0b100, 3)), Some('C'));
        assert_eq!(*htree.lookup_symbol(Code::new(0b101, 3)), Some('D'));
        assert_eq!(*htree.lookup_symbol(Code::new(0b110, 3)), Some('E'));
        assert_eq!(*htree.lookup_symbol(Code::new(0b00, 2)), Some('F'));
        assert_eq!(*htree.lookup_symbol(Code::new(0b1110, 4)), Some('G'));
        assert_eq!(*htree.lookup_symbol(Code::new(0b1111, 4)), Some('H'));
    }
}
