use sl_std::{bytestream::ByteStream, safe_casts::Plain};

use super::Error;

#[rustfmt::skip]
pub const ORDER_TO_MATRIX_INDEX: [(i32, i32); 64] = [
    (0,0),
    (0,1), (1,0),         
    (2,0), (1,1), (0,2),
    (0,3), (1,2), (2,1), (3,0),
    (4,0), (3,1), (2,2), (1,3), (0,4),
    (0,5), (1,4), (2,3), (3,2), (4,1), (5,0),
    (6,0), (5,1), (4,2), (3,3), (2,4), (1,5), (0,6),
    (0,7), (1,6), (2,5), (3,4), (4,3), (5,2), (6,1), (7,0),
    (7,1), (6,2), (5,3), (4,4), (3,5), (2,6), (1,7),
    (2,7), (3,6), (4,5), (5,4), (6,3), (7,2),
    (7,3), (6,4), (5,5), (4,6), (3,7),
    (4,7), (5,6), (6,5), (7,4),
    (7,5), (6,6), (5,7),
    (6,7), (7,6),
    (7,7)
];

#[rustfmt::skip]
pub const MATRIX_INDEX_TO_ORDER: [[i32; 8]; 8] = [
    [  0,  1,  5,  6, 14, 15, 27, 28 ],
    [  2,  4,  7, 13, 16, 26, 29, 42 ],
    [  3,  8, 12, 17, 25, 30, 41, 43 ],
    [  9, 11, 18, 24, 31, 40, 44, 53 ],
    [ 10, 19, 23, 32, 39, 45, 52, 54 ],
    [ 20, 22, 33, 38, 46, 51, 55, 60 ],
    [ 21, 34, 37, 47, 50, 56, 59, 61 ],
    [ 35, 36, 48, 49, 57, 58, 62, 63 ]
];

const MAX_QUANTIZATION_TABLES: usize = 4;

#[derive(Clone, Debug, Default)]
pub struct QuantizationTables {
    /// Stored in a `Box` to not use too much stack
    tables: Box<[Option<QuantizationTable>; MAX_QUANTIZATION_TABLES]>,
}

#[derive(Clone, Copy, Debug)]
enum Precision {
    U8,
    U16,
}

pub type QuantizationTable = [u16; 64];

impl QuantizationTables {
    #[must_use]
    pub fn get(&self, index: u8) -> Result<&QuantizationTable, Error> {
        self.tables[index as usize]
            .as_ref()
            .ok_or(Error::UndefinedQuantizationTable)
    }

    pub fn add_tables(&mut self, tables: &[u8]) -> Result<(), Error> {
        let mut byte_stream = ByteStream::new(tables);

        // There might be multiple quantization tables stored after one another
        while !byte_stream.is_empty() {
            let pq_tq = byte_stream.next_byte().ok_or(Error::BadChunk)?;
            let precision = match pq_tq & 0xF0 {
                0 => Precision::U8,
                1 => Precision::U16,
                _ => return Err(Error::BadQuantizationTable),
            };

            let destination = pq_tq as usize & 0x0F;
            if self.tables.len() <= destination {
                return Err(Error::BadQuantizationTable);
            }

            // Read 64 elements whose size is specified by "precision"
            let quantization_table: [u16; 64] = match precision {
                Precision::U8 => {
                    // Read 64 bytes and extend them to u16s
                    let table = byte_stream.next_chunk::<64>().ok_or(Error::BadChunk)?;

                    let mut extended_table: [u16; 64] = [0; 64];
                    for (src, dst) in table.iter().zip(extended_table.iter_mut()) {
                        *dst = *src as u16;
                    }

                    extended_table
                },
                Precision::U16 => {
                    let mut table: [u16; 64] = byte_stream
                        .next_chunk::<128>()
                        .ok_or(Error::BadChunk)?
                        .cast();

                    // Values are stored in big-endian, so if we're on a little-endian platform then we need to swap
                    // all bytes
                    if cfg!(target_endian = "little") {
                        for elem in &mut table {
                            *elem = elem.swap_bytes()
                        }
                    }

                    table
                },
            };

            if quantization_table.iter().any(|&e| e == 0) {
                return Err(Error::ZeroInQuantizationTable);
            }

            self.tables[destination] = Some(quantization_table);
        }

        Ok(())
    }
}
