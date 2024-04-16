use std::num::{NonZero, NonZeroU16};

use super::{bit_reader::BitReader, Error};

const MAX_NUM_HUFFMAN_TABLES: usize = 32;

#[derive(Clone, Default)]
pub struct HuffmanTables {
    tables: [Option<HuffmanTable>; MAX_NUM_HUFFMAN_TABLES],
}

/// A huffman table for decoding symbols
///
/// Implemented as in <https://commandlinefanatic.com/cgi-bin/showarticle.cgi?article=art007>
#[derive(Clone, Debug)]
pub struct HuffmanTable {
    lookup_table: Box<[HuffmanTableEntry; u16::MAX as usize]>,
}

#[derive(Clone, Copy, Debug)]
pub struct HuffmanLookupResult {
    symbol: u8,
    consumed_bits: NonZero<u32>,
}

impl HuffmanTable {
    fn insert_symbol(&mut self, code: u16, mask: NonZeroU16, symbol: u8) {
        let base = code << mask.trailing_zeros();
        let repeat_count = (1 << mask.trailing_zeros()) - 1;

        let entry = HuffmanTableEntry { mask, symbol };
        for i in 0..repeat_count {
            self.lookup_table[(base + i) as usize] = entry;
        }
    }

    #[must_use]
    pub fn lookup_code(&self, code: u16) -> HuffmanLookupResult {
        let entry = self.lookup_table[code as usize];
        HuffmanLookupResult {
            symbol: entry.symbol,
            consumed_bits: entry.mask.count_ones(),
        }
    }

    #[must_use]
    pub fn lookup_code_from_reader(&self, reader: &mut BitReader<'_>) -> u8 {
        let result = self.lookup_code(reader.peek_u16());
        reader.advance(result.consumed_bits.get() as usize);

        result.symbol
    }
}

impl Default for HuffmanTable {
    fn default() -> Self {
        // All codes must be initialized to zero to make the logic in insert_symbol work
        let initial_symbol = HuffmanTableEntry {
            mask: NonZeroU16::new(1).expect("1 is not zero"),
            symbol: 0,
        };

        Self {
            lookup_table: Box::new([initial_symbol; u16::MAX as usize]),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct HuffmanTableEntry {
    /// Masks the relevant input bits, left aligned.
    ///
    /// For example, if the symbol code has a length of 3
    /// then the mask is `0b11100000`.
    ///
    /// Can never be zero because a huffman code can never have length zero
    mask: NonZeroU16,

    /// The decoded code
    symbol: u8,
}

impl HuffmanTables {
    #[must_use]
    pub fn get(&self, index: u8) -> Result<&HuffmanTable, Error> {
        self.tables[index as usize]
            .as_ref()
            .ok_or(Error::UndefinedHuffmanTable)
    }

    pub fn add_table(&mut self, bytes: &[u8]) -> Result<(), Error> {
        // First byte is the table id
        let table_id = *bytes.get(0).ok_or(Error::BadHuffmanTable)? as usize;

        if self.tables.len() <= table_id {
            return Err(Error::BadHuffmanTable);
        }

        // Next 16 bytes are the counts for each code length
        let mut counts = [0; 16];
        counts.copy_from_slice(bytes.get(1..17).ok_or(Error::BadHuffmanTable)?);

        // Remaining bytes are the data values to be mapped
        // Build the Huffman map of (length, code) -> value
        let mut bytes = bytes[17..].iter();

        let mut code: u16 = 0;
        let mut table = HuffmanTable::default();
        for code_length in 1..=16 {
            // This computes mask as an integer whose first code_length + 1 bits are 1 and 0 otherwise
            let mask =
                NonZeroU16::new(!((1 << (u16::BITS - code_length)) - 1)).expect("cannot be zero");
            let n_codes_with_this_length = counts[code_length as usize - 1];

            for _ in 0..n_codes_with_this_length {
                let symbol = *bytes.next().ok_or(Error::BadHuffmanTable)?;
                table.insert_symbol(code, mask, symbol);
                code += 1;
            }
            code <<= 1;
        }

        self.tables[table_id] = Some(table);

        Ok(())
    }
}
