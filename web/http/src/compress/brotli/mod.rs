//! Implements the [Brotli](https://datatracker.ietf.org/doc/html/rfc7932) compression algorithm

pub mod huffman;

use crate::compress::bit_reader::{BitReader, BitReaderError};

use self::huffman::{Bits, Code, HuffmanTree};

#[derive(Debug)]
pub enum BrotliError {
    UnexpectedEOF,
    InvalidFormat,
    InvalidSymbol,
    MismatchedChecksum,
    /// A run-length encoded value decoded to more symbols than expected
    RunlengthEncodingExceedsExpectedSize,
    /// A complex prefix code contained less than two nonzero code lengths
    NotEnoughCodeLengths,
    SymbolNotFound,
}

impl From<BitReaderError> for BrotliError {
    fn from(from: BitReaderError) -> Self {
        match from {
            BitReaderError::UnexpectedEOF => Self::UnexpectedEOF,
            BitReaderError::TooLargeRead => panic!("trying to read too many bits at once"),
            BitReaderError::UnalignedRead => panic!("unaligned read"),
        }
    }
}

const NBLTYPESI: usize = 0;
const NBLTYPESL: usize = 1;
const NBLTYPESD: usize = 2;

// https://www.rfc-editor.org/rfc/rfc7932#section-10
pub fn decode(source: &[u8]) -> Result<Vec<u8>, BrotliError> {
    let mut reader = BitReader::new(source);
    let mut result = vec![];

    // Read the Stream Header (which only contains the sliding window size)
    // https://www.rfc-editor.org/rfc/rfc7932#section-9.1
    let wbits = if reader.read_bits::<u8>(1).map_err(BrotliError::from)? == 0b0 {
        16
    } else {
        let n2 = reader.read_bits::<u8>(3).map_err(BrotliError::from)?;
        if n2 == 0b000 {
            let n3 = reader.read_bits::<u8>(3).map_err(BrotliError::from)?;
            if n3 == 0b000 {
                17
            } else {
                8 + n3
            }
        } else {
            17 + n2
        }
    };

    let window_size = (1 << wbits) - 16;

    let mut is_last = false;

    while !is_last {
        // read meta block header
        // read ISLAST bit
        is_last = reader.read_single_bit().map_err(BrotliError::from)?;

        if is_last {
            // read ISLASTEMPTY bit
            if reader.read_single_bit().map_err(BrotliError::from)? {
                break;
            }
        }

        // read MNIBBLES
        let mnibbles = match reader.read_bits::<u8>(2).map_err(BrotliError::from)? {
            0b11 => 0,
            0b00 => 4,
            0b01 => 5,
            0b10 => 6,
            _ => unreachable!(),
        };

        let mlen = if mnibbles == 0 {
            // verify reserved bit is zero
            if reader.read_single_bit().map_err(BrotliError::from)? {
                return Err(BrotliError::InvalidFormat);
            }

            // read MSKIPLEN
            todo!("do empty blocks even occur in the real word");
        // skip any bits up to the next byte boundary
        } else {
            // read MLEN
            reader
                .read_bits::<u32>(4 * mnibbles)
                .map_err(BrotliError::from)? as usize
                + 1
        };

        if !is_last {
            let is_uncompressed = reader.read_single_bit().map_err(BrotliError::from)?;

            if is_uncompressed {
                reader.align_to_byte_boundary();

                let mut buffer = vec![0; mlen];
                reader.read_bytes(&mut buffer).map_err(BrotliError::from)?;
                result.extend(buffer);
                continue;
            }
        }

        let mut num_blocks = [0; 3];
        let mut block_type = [0; 3];
        let mut block_length = [0; 3];

        for i in 0..3 {
            num_blocks[i] = decode_blocknum(&mut reader)?;

            println!("nbltypes {}", num_blocks[i]);
            if num_blocks[i] > 2 {
                todo!();
            } else {
                block_type[i] = 0;
                block_length[i] = 16777216;
            }
        }

        // read NPOSTFIX and NDIRECT
        let npostfix = reader.read_bits::<u8>(2).map_err(BrotliError::from)?;

        // number of direct-instance codes
        let ndirect = reader.read_bits::<u8>(4).map_err(BrotliError::from)? << npostfix;

        let mut context_modes_for_literal_block_types =
            Vec::with_capacity(num_blocks[NBLTYPESL] as usize);
        for _ in 0..num_blocks[NBLTYPESL] {
            let context_mode = reader.read_bits::<u8>(2).map_err(BrotliError::from)?;
            context_modes_for_literal_block_types.push(context_mode);
        }

        // read NTREES
        let ntreesl = decode_blocknum(&mut reader)?;
        let literal_cmap = if ntreesl >= 2 {
            // parse context map literals
            decode_context_map(&mut reader, ntreesl, 64 * num_blocks[NBLTYPESL] as usize)?
        } else {
            // fill cmapl with zeros
            vec![0; 64 * num_blocks[NBLTYPESL] as usize]
        };

        let ntreesd = decode_blocknum(&mut reader)?;
        let distance_cmap = if ntreesd >= 2 {
            // decode_context_map(&mut reader, ntreesd, 4 * num_blocks[NBLTYPESD] as usize)?;
            decode_context_map(&mut reader, ntreesl, 4 * num_blocks[NBLTYPESD] as usize)?
        } else {
            // fill cmapd with zeros
            vec![0; 4 * num_blocks[NBLTYPESD] as usize]
        };

        // Read literal prefix codes
        let mut literal_prefix_codes = Vec::with_capacity(ntreesl as usize);
        for _ in 0..ntreesl {
            literal_prefix_codes.push(read_prefix_code(&mut reader, 256)?);
        }
    }
    Ok(result)
}

fn read_prefix_code(
    reader: &mut BitReader,
    alphabet_size: usize,
) -> Result<HuffmanTree<Bits<u32>>, BrotliError> {
    let alphabet_width = 16 - (alphabet_size as u16 - 1).leading_zeros() as u8;

    let ident = reader.read_bits::<u8>(2).map_err(BrotliError::from)?;
    let mut symbols_raw = vec![];

    let huffmantree = if ident == 1 {
        // Simple prefix code
        let nsym = reader.read_bits::<u8>(2).map_err(BrotliError::from)? + 1;

        // read nsym symbols
        for _ in 0..nsym {
            let symbol_raw = reader
                .read_bits::<u32>(alphabet_width)
                .map_err(BrotliError::from)?;
            if symbol_raw >= alphabet_size as u32 {
                return Err(BrotliError::InvalidSymbol);
            }
            symbols_raw.push(symbol_raw);
        }

        // TODO we should check for duplicate symbols here

        let lengths = match nsym {
            0 => vec![0],
            1 => {
                symbols_raw.sort();
                vec![1, 1]
            },
            3 => {
                symbols_raw[1..].sort();
                vec![1, 2, 2]
            },
            4 => {
                if reader.read_single_bit().map_err(BrotliError::from)? {
                    symbols_raw[2..].sort();
                    vec![1, 2, 3, 3]
                } else {
                    symbols_raw.sort();
                    vec![2, 2, 2, 2]
                }
            },
            _ => unreachable!(),
        };

        // Associate the symbols with their bit length. We didn't to this earlier so
        // we could sort the symbols without worrying about bit length.
        let symbols: Vec<Bits<u32>> = symbols_raw
            .into_iter()
            .map(|raw_symbol| Bits::new(raw_symbol, alphabet_width as usize))
            .collect();

        HuffmanTree::new_infer_codes(&symbols, &lengths)
    } else {
        let hskip = ident as usize;

        // Complex prefix code
        let symbols = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17];
        let mut code_lengths = [0; 18];
        let mut checksum = 0;

        // the first hskip code lengths are assumed to be 0
        for i in hskip..18 {
            // Code lengths are encoded in a variable length code
            // Symbol   Code
            // ------   ----
            // 0          00
            // 1        0111
            // 2         011
            // 3          10
            // 4          01
            // 5        1111
            let code_length = match reader.read_bits::<u8>(2).map_err(BrotliError::from)? {
                0b00 => 0,
                0b10 => 3,
                0b01 => 4,
                0b11 => {
                    if reader.read_single_bit().map_err(BrotliError::from)? {
                        if reader.read_single_bit().map_err(BrotliError::from)? {
                            5
                        } else {
                            1
                        }
                    } else {
                        2
                    }
                },
                _ => unreachable!(),
            };
            code_lengths[i] = code_length;

            if code_length != 0 {
                checksum += 32 >> code_length;

                if checksum == 32 {
                    break;
                }
            }
        }

        if code_lengths.iter().filter(|x| **x != 0).count() >= 2 {
            if checksum != 32 {
                return Err(BrotliError::MismatchedChecksum);
            }
        }

        // Code lengths are not given in the correct order but our huffmantree implementation requires that
        code_lengths[..5].rotate_right(1);
        code_lengths[6..].rotate_left(1);
        code_lengths[7..17].rotate_left(1);

        let code_length_encoding = HuffmanTree::new_infer_codes(&symbols, &code_lengths);

        let mut checksum = 0;
        let mut symbol_lengths = vec![0; alphabet_size];
        let mut previous_nonzero_code = None;
        let mut previous_repeat_count = None;
        let mut i = 0;

        'read_length_codes: while i < alphabet_size {
            let symbol_length_code = *code_length_encoding
                .lookup_incrementally(reader)
                .map_err(|_| BrotliError::UnexpectedEOF)?
                .ok_or(BrotliError::SymbolNotFound)?;

            match symbol_length_code {
                0..=15 => {
                    symbol_lengths[i] = symbol_length_code as usize;
                    i += 1;

                    if symbol_length_code != 0 {
                        checksum += 32768 >> symbol_length_code;
                        previous_nonzero_code = Some(symbol_length_code);

                        if checksum == 32768 {
                            break 'read_length_codes;
                        }
                    }

                    previous_repeat_count = None;
                },
                16 => {
                    let extra_bits = reader.read_bits::<usize>(2).map_err(BrotliError::from)?;

                    let repeat_for = match previous_repeat_count {
                        Some((16, previous_repetitions)) => {
                            // There was a 16 previously, update repeat count
                            let new_repeat = 4 * (previous_repetitions - 2) + 3 + extra_bits;
                            new_repeat - previous_repetitions
                        },
                        _ => {
                            // The previous length code was not a 16
                            3 + extra_bits as usize
                        },
                    };

                    // Check which element we should be repeating
                    let to_repeat = match previous_nonzero_code {
                        Some(code) => code,
                        None => 8,
                    };

                    // Make sure to not exceed the alphabet size
                    if i + repeat_for > alphabet_size {
                        return Err(BrotliError::RunlengthEncodingExceedsExpectedSize);
                    }

                    i += repeat_for;
                    for _ in 0..repeat_for {
                        symbol_lengths[i] = to_repeat;

                        checksum += 32768 >> to_repeat;
                    }
                    if checksum == 32768 {
                        break 'read_length_codes;
                    }

                    previous_repeat_count = Some((16, repeat_for));
                },
                17 => {
                    let extra_bits = reader.read_bits::<usize>(3).map_err(BrotliError::from)?;

                    let repeat_for = match previous_repeat_count {
                        Some((17, previous_repetitions)) => {
                            // There was a 16 previously, update repeat count
                            let new_repeat = 8 * (previous_repetitions - 2) + 3 + extra_bits;
                            new_repeat - previous_repetitions
                        },
                        _ => {
                            // The previous length code was not a 16
                            3 + extra_bits as usize
                        },
                    };

                    // Make sure to not exceed the alphabet size
                    if i + repeat_for > alphabet_size {
                        return Err(BrotliError::RunlengthEncodingExceedsExpectedSize);
                    }

                    i += repeat_for;

                    previous_repeat_count = Some((17, repeat_for));
                },
                _ => unreachable!(), // we defined the possible symbols above
            }
        }

        if checksum != 32768 {
            return Err(BrotliError::MismatchedChecksum);
        }

        // Every complex prefix code must contain at least two nonzero code lengths
        if symbol_lengths.iter().filter(|x| **x != 0).count() < 2 {
            return Err(BrotliError::NotEnoughCodeLengths);
        }

        let symbols: Vec<Bits<u32>> = (0..alphabet_size as u32)
            .map(|val| Bits::new(val, alphabet_size))
            .collect();
        HuffmanTree::new_infer_codes(&symbols, &symbol_lengths)
    };
    Ok(huffmantree)
}

fn decode_blocknum(reader: &mut BitReader) -> Result<u8, BrotliError> {
    if reader.read_single_bit().map_err(BrotliError::from)? {
        let num_extrabits = reader.read_bits::<u8>(3).map_err(BrotliError::from)?;

        if num_extrabits > 7 {
            return Err(BrotliError::InvalidFormat);
        }

        let extra = reader
            .read_bits::<u8>(num_extrabits)
            .map_err(BrotliError::from)?;
        Ok((1 << num_extrabits) + 1 + extra)
    } else {
        Ok(1)
    }
}

/// https://www.rfc-editor.org/rfc/rfc7932#section-7.3
fn decode_context_map(
    reader: &mut BitReader,
    num_trees: u8,
    size: usize,
) -> Result<Vec<u8>, BrotliError> {
    let rle_max = match reader.read_single_bit().map_err(BrotliError::from)? {
        false => 0,
        true => reader.read_bits::<u8>(4).map_err(BrotliError::from)? + 1,
    };

    let prefix_code = read_prefix_code(reader, (num_trees + rle_max) as usize)?;

    let mut context_map = Vec::with_capacity(size);

    // Values are encoded using a combination of prefix coding and run-length encoding
    // The alphabet looks like this (taken from the specification):
    //
    // 0: value zero
    // 1: repeat a zero 2 to 3 times, read 1 bit for repeat length
    // 2: repeat a zero 4 to 7 times, read 2 bits for repeat length
    // ...
    // RLEMAX: repeat a zero (1 << RLEMAX) to (1 << (RLEMAX+1))-1
    // 		times, read RLEMAX bits for repeat length
    // RLEMAX + 1: value 1
    // ...
    // RLEMAX + NTREES - 1: value NTREES - 1
    while context_map.len() < size {
        let symbol = prefix_code
            .lookup_incrementally(reader)
            .map_err(|_| BrotliError::UnexpectedEOF)?
            .ok_or(BrotliError::SymbolNotFound)?
            .val();

        if symbol <= rle_max as u32 {
            // This is a run-length encoded value

            // Casting to u8 here is safe because rle_max can never exceed 255
            let extra_bits = reader
                .read_bits::<u32>(symbol as u8)
                .map_err(BrotliError::from)?;
            let repeat_for = (1 << symbol) + extra_bits as usize;

            if context_map.len() + repeat_for > size {
                return Err(BrotliError::RunlengthEncodingExceedsExpectedSize);
            }

            context_map.reserve(repeat_for);
            for _ in 0..repeat_for {
                context_map.push(0);
            }
        } else {
            context_map.push((symbol - rle_max as u32) as u8);
        }
    }

    // Check whether we need to do an inverse move-to-front transform
    if reader.read_single_bit().map_err(BrotliError::from)? {
        inverse_move_to_front_transform(&mut context_map);
    }

    Ok(context_map)
}

fn inverse_move_to_front_transform(data: &mut [u8]) {
    let mut mtf = [0; 256];

    for i in 0..256 {
        mtf[i] = i as u8;
    }

    for i in 0..data.len() {
        let index = data[i] as usize;
        let value = mtf[index];

        data[i] = value;

        // TODO i feel like we can make this faster, perhaps with some sort of queue?
        for j in (1..index + 1).rev() {
            mtf[j] = mtf[j - 1];
        }
        mtf[0] = value;
    }
}
