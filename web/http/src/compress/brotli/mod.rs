//! Implements the [Brotli](https://datatracker.ietf.org/doc/html/rfc7932) compression algorithm

pub mod dictionary;
pub mod huffman;

use crate::compress::{
    bit_reader::{BitReader, BitReaderError},
    brotli::huffman::{Bits, HuffmanTree},
    ringbuffer::RingBuffer,
};
use std::cmp::min;

macro_rules! update_block_type_and_count {
    ($btype: ident, $btype_tree: ident, $blen: ident, $blen_tree: ident, $btype_prev: ident, $nbl: ident, $reader: expr) => {
        let btype_code = $btype_tree
            .as_ref()
            .unwrap()
            .lookup_incrementally($reader)
            .map_err(|_| BrotliError::UnexpectedEOF)?
            .ok_or(BrotliError::SymbolNotFound)?
            .val();

        let block_type = match btype_code {
            0 => $btype_prev,
            1 => ($btype + 1) % $nbl,
            _ => btype_code - 2,
        };

        $btype_prev = $btype;
        $btype = block_type;

        let blen_code = $blen_tree
            .as_ref()
            .unwrap()
            .lookup_incrementally($reader)
            .map_err(|_| BrotliError::UnexpectedEOF)?
            .ok_or(BrotliError::SymbolNotFound)?
            .val();

        let (base, num_extra_bits) = decode_blocklen(blen_code);
        let extra_bits = $reader
            .read_bits::<usize>(num_extra_bits as u8)
            .map_err(BrotliError::from)?;
        $blen = base + extra_bits;
    };
}

#[rustfmt::skip]
const LUT0: [u8; 256] = [
     0,  0,  0,  0,  0,  0,  0,  0,  0,  4,  4,  0,  0,  4,  0,  0,
     0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,
     8, 12, 16, 12, 12, 20, 12, 16, 24, 28, 12, 12, 32, 12, 36, 12,
    44, 44, 44, 44, 44, 44, 44, 44, 44, 44, 32, 32, 24, 40, 28, 12,
    12, 48, 52, 52, 52, 48, 52, 52, 52, 48, 52, 52, 52, 52, 52, 48,
    52, 52, 52, 52, 52, 48, 52, 52, 52, 52, 52, 24, 12, 28, 12, 12,
    12, 56, 60, 60, 60, 56, 60, 60, 60, 56, 60, 60, 60, 60, 60, 56,
    60, 60, 60, 60, 60, 56, 60, 60, 60, 60, 60, 24, 12, 28, 12,  0,
     0,  1,  0,  1,  0,  1,  0,  1,  0,  1,  0,  1,  0,  1,  0,  1,
     0,  1,  0,  1,  0,  1,  0,  1,  0,  1,  0,  1,  0,  1,  0,  1,
     0,  1,  0,  1,  0,  1,  0,  1,  0,  1,  0,  1,  0,  1,  0,  1,
     0,  1,  0,  1,  0,  1,  0,  1,  0,  1,  0,  1,  0,  1,  0,  1,
     2,  3,  2,  3,  2,  3,  2,  3,  2,  3,  2,  3,  2,  3,  2,  3,
     2,  3,  2,  3,  2,  3,  2,  3,  2,  3,  2,  3,  2,  3,  2,  3,
     2,  3,  2,  3,  2,  3,  2,  3,  2,  3,  2,  3,  2,  3,  2,  3,
     2,  3,  2,  3,  2,  3,  2,  3,  2,  3,  2,  3,  2,  3,  2,  3,
];

#[rustfmt::skip]
const LUT1: [u8; 256] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 1,
    1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1,
    1, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
    3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 1, 1, 1, 1, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2
];

#[rustfmt::skip]
const LUT2: [u8; 256] = [
    0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
    3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
    3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
    3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
    4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
    4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
    4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
    4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
    5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
    5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
    5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
    6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 7
];

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
    InvalidDictionaryReferenceLength,
    InvalidTransformID,
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

// https://www.rfc-editor.org/rfc/rfc7932#section-10
pub fn decode(source: &[u8]) -> Result<Vec<u8>, BrotliError> {
    let mut reader = BitReader::new(source);

    // The stream initially contains two zero bytes since decoding relies on the "last two uncompressed bytes", which are initally 0
    // These bytes are removed later
    let mut output_stream = vec![0, 0];

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
    let mut past_distances = RingBuffer::new(vec![16, 15, 11, 4]);

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
        println!("mlen {mlen}");

        if !is_last {
            let is_uncompressed = reader.read_single_bit().map_err(BrotliError::from)?;

            if is_uncompressed {
                reader.align_to_byte_boundary();

                let mut buffer = vec![0; mlen];
                reader.read_bytes(&mut buffer).map_err(BrotliError::from)?;
                output_stream.extend(buffer);
                continue;
            }
        }

        let (nbl_types_l, htree_btype_l, htree_blen_l, mut blen_l) = decode_blockdata(&mut reader)?;

        let (nbl_types_i, htree_btype_i, htree_blen_i, mut blen_i) = decode_blockdata(&mut reader)?;

        let (nbl_types_d, htree_btype_d, htree_blen_d, mut blen_d) = decode_blockdata(&mut reader)?;

        // read NPOSTFIX and NDIRECT
        let npostfix = reader.read_bits::<usize>(2).map_err(BrotliError::from)?;
        let ndirect = reader.read_bits::<usize>(4).map_err(BrotliError::from)? << npostfix;

        let mut context_modes_for_literal_block_types = Vec::with_capacity(nbl_types_l);
        for _ in 0..nbl_types_l {
            let context_mode = reader.read_bits::<u8>(2).map_err(BrotliError::from)?;
            context_modes_for_literal_block_types.push(context_mode);
        }

        // read NTREES
        let ntreesl = decode_blocknum(&mut reader)?;

        let cmap_l = if ntreesl >= 2 {
            // parse context map literals
            decode_context_map(&mut reader, ntreesl, 64 * nbl_types_l)?
        } else {
            // fill cmapl with zeros
            vec![0; 64 * nbl_types_l]
        };

        let ntreesd = decode_blocknum(&mut reader)?;
        let cmap_d = if ntreesd >= 2 {
            // decode_context_map(&mut reader, ntreesd, 4 * nbl_types_d as usize)?;
            decode_context_map(&mut reader, ntreesd, 4 * nbl_types_d)?
        } else {
            // fill cmapd with zeros
            vec![0; 4 * nbl_types_d]
        };

        // Read literal prefix codes
        let mut htree_l = Vec::with_capacity(ntreesl as usize);

        for _ in 0..ntreesl {
            htree_l.push(read_prefix_code(&mut reader, 256)?);
        }

        // Read insert-and-copy lengths
        let mut htree_i = Vec::with_capacity(nbl_types_i);
        for _ in 0..nbl_types_i {
            htree_i.push(read_prefix_code(&mut reader, 704)?);
        }

        // Read literal prefix codes
        let mut htree_d = Vec::with_capacity(ntreesd as usize);
        for _ in 0..ntreesd {
            htree_d.push(read_prefix_code(
                &mut reader,
                16 + ndirect + (48 << npostfix),
            )?);
        }

        // Parse the meta block data
        let mut uncompressed_bytes_this_meta_block = 0;

        let mut btype_l = 0;
        let mut btype_i = 0;
        let mut btype_d = 0;
        let mut previous_btype_l = 1;
        let mut previous_btype_i = 1;
        let mut previous_btype_d = 1;

        'decode_loop: loop {
            if blen_i == 0 {
                println!("UPDATE BLENI");
                update_block_type_and_count!(
                    btype_i,
                    htree_btype_i,
                    blen_i,
                    htree_blen_i,
                    previous_btype_i,
                    nbl_types_i,
                    &mut reader
                );
            }

            blen_i -= 1;

            let insert_and_copy_length_code = htree_i[btype_i]
                .lookup_incrementally(&mut reader)
                .map_err(|_| BrotliError::UnexpectedEOF)?
                .ok_or(BrotliError::SymbolNotFound)?
                .val();
            println!("====================== {}", output_stream.len() - 2);
            // println!("bit reader pos after ic length code {} {}", reader.byte_ptr, reader.bit_ptr);
            // println!("insert code {insert_and_copy_length_code}");

            // if insert_and_copy_length_code == 19 {
            //     todo!();
            // }

            // TODO apparently distance is implicit zero if the symbol is < 128
            let distance_is_implicit_zero = insert_and_copy_length_code < 128;

            let (insert_length, copy_length) =
                decode_insert_and_copy_length_code(insert_and_copy_length_code);

            // println!("the length codes: {insert_length} {copy_length}");
            let ilen = read_insert_length_code(&mut reader, insert_length)?;
            let clen = read_copy_length_code(&mut reader, copy_length)?;

            for _ in 0..ilen {
                // println!("bit reader pos during literals {} {}", reader.byte_ptr, reader.bit_ptr);
                if blen_l == 0 {
                    println!("UPDATE BLENL XXXXXXXXXXXXXXXXXXXXXXXXXXX");
                    update_block_type_and_count!(
                        btype_l,
                        htree_btype_l,
                        blen_l,
                        htree_blen_l,
                        previous_btype_l,
                        nbl_types_l,
                        &mut reader
                    );
                }
                blen_l -= 1;

                let context_mode = context_modes_for_literal_block_types[btype_l];
                // println!("Last two bytes: {:?}", &output_stream[output_stream.len() - 2..]);
                let cidl = decode_literal_context_id(
                    context_mode,
                    &output_stream[output_stream.len() - 2..].try_into().unwrap(),
                );

                println!("CID {cidl} context mode {context_mode}");
                println!("bits pos {} {}", reader.byte_ptr, reader.bit_ptr);
                println!("tree index {}", cmap_l[64 * btype_l + cidl as usize]);
                println!("btype l {btype_l}");

                if reader.byte_ptr == 595 && reader.bit_ptr == 4 {
                    println!(
                        "tree {:?}",
                        htree_l[cmap_l[64 * btype_l + cidl as usize] as usize]
                    );
                }

                let literal_symbol = htree_l[cmap_l[64 * btype_l + cidl as usize] as usize]
                    .lookup_incrementally(&mut reader)
                    .map_err(|_| BrotliError::UnexpectedEOF)?
                    .ok_or(BrotliError::SymbolNotFound)
                    .unwrap();

                println!("emitting {}", literal_symbol.val() as u8);
                output_stream.push(literal_symbol.val() as u8);
                uncompressed_bytes_this_meta_block += 1;
            }

            if uncompressed_bytes_this_meta_block == mlen {
                break 'decode_loop;
            }
            println!("done parsing literals");
            // Distances larger that max_distance can occur, those are static dictionary references
            // We subtract two because the output contains two leading 0 bytes which are not part of the stream
            let max_distance = min(window_size, output_stream.len() - 2);
            let distance = if distance_is_implicit_zero {
                // println!("implicit 0 distance");
                *past_distances.nth_last(0)
            } else {
                if blen_d == 0 {
                    // println!("update d type ");
                    update_block_type_and_count!(
                        btype_d,
                        htree_btype_d,
                        blen_d,
                        htree_blen_d,
                        previous_btype_d,
                        nbl_types_d,
                        &mut reader
                    );
                }
                blen_d -= 1;

                let cidd = decode_distance_context_id(clen);
                let distance_code = htree_d[cmap_d[4 * btype_d + cidd] as usize]
                    .lookup_incrementally(&mut reader)
                    .map_err(|_| BrotliError::UnexpectedEOF)?
                    .ok_or(BrotliError::SymbolNotFound)?
                    .val();
                // println!("distance code: {distance_code}");
                let distance = distance_short_code_substitution(
                    distance_code,
                    &past_distances,
                    npostfix,
                    ndirect,
                    &mut reader,
                )?;

                // Dictionary references, 0 distances and a few transformations are not pushed
                if distance_code != 0 && distance < max_distance {
                    past_distances.push(distance);
                }
                distance
            };
            println!("done parsing distance");
            println!("distance {distance} {clen}");
            // println!("max ddist {max_distance}");
            if distance <= max_distance {
                // println!("distance ref {distance} {clen}");

                // resolve distance
                let copy_base = output_stream.len() - distance;

                // References can be longer than the data that is actually available.
                // In this case, the reference wraps around and copies the beginning twice
                let mut literals_remaining = clen;
                let bytes_to_copy_at_once = min(clen, output_stream.len() - copy_base);

                while literals_remaining > bytes_to_copy_at_once {
                    output_stream.extend_from_within(copy_base..copy_base + bytes_to_copy_at_once);
                    literals_remaining -= bytes_to_copy_at_once;
                }
                // println!("emitting {:?} from {copy_base}, {clen}", &output_stream[copy_base..copy_base+clen]);
                output_stream.extend_from_within(copy_base..copy_base + literals_remaining);
                uncompressed_bytes_this_meta_block += clen;
            } else {
                let dict_word = dictionary::lookup(distance - max_distance - 1, clen)?;
                uncompressed_bytes_this_meta_block += dict_word.len();
                println!("emitting {dict_word:?} from dict");
                output_stream.extend(dict_word);
            }

            if uncompressed_bytes_this_meta_block >= mlen {
                break;
            }
        }
        // println!("DONE READING BLOCK");

        if is_last {
            break;
        }
    }
    Ok(output_stream[2..].to_vec())
}

fn read_prefix_code(
    reader: &mut BitReader,
    alphabet_size: usize,
) -> Result<HuffmanTree<Bits<usize>>, BrotliError> {
    let alphabet_width = 16 - (alphabet_size as u16 - 1).leading_zeros() as u8;

    let ident = reader.read_bits::<u8>(2).map_err(BrotliError::from)?;
    let mut symbols_raw = vec![];

    let huffmantree = if ident == 1 {
        // Simple prefix code
        let nsym = reader.read_bits::<u8>(2).map_err(BrotliError::from)? + 1;

        // read nsym symbols
        for _ in 0..nsym {
            let symbol_raw = reader
                .read_bits::<usize>(alphabet_width)
                .map_err(BrotliError::from)?;

            // Reject symbol if its not within the alphabet
            if symbol_raw >= alphabet_size {
                return Err(BrotliError::InvalidSymbol);
            }
            symbols_raw.push(symbol_raw);
        }

        // TODO we should check for duplicate symbols here

        let lengths = match nsym {
            1 => vec![0],
            2 => {
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
            _ => unreachable!("Invalid NSYM value: {nsym}"),
        };

        // Associate the symbols with their bit length. We didn't to this earlier so
        // we could sort the symbols without worrying about bit length.
        let symbols: Vec<Bits<usize>> = symbols_raw
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

                    for j in 0..repeat_for {
                        symbol_lengths[i + j] = to_repeat;

                        checksum += 32768 >> to_repeat;
                    }
                    i += repeat_for;
                    if checksum == 32768 {
                        break 'read_length_codes;
                    }

                    previous_repeat_count = Some((16, repeat_for));
                },
                17 => {
                    let extra_bits = reader.read_bits::<usize>(3).map_err(BrotliError::from)?;

                    let (repeat_for, total_repetitions) = match previous_repeat_count {
                        Some((17, previous_repetitions)) => {
                            // There was a 16 previously, update repeat count
                            let new_repeat = 8 * (previous_repetitions - 2) + 3 + extra_bits;
                            (new_repeat - previous_repetitions, new_repeat)
                        },
                        _ => {
                            // The previous length code was not a 17
                            (3 + extra_bits as usize, 3 + extra_bits as usize)
                        },
                    };

                    // Make sure to not exceed the alphabet size
                    if i + repeat_for > alphabet_size {
                        return Err(BrotliError::RunlengthEncodingExceedsExpectedSize);
                    }

                    i += repeat_for;

                    previous_repeat_count = Some((17, total_repetitions));
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

        let symbols: Vec<Bits<usize>> = (0..alphabet_size)
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

        if symbol <= rle_max as usize {
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
            context_map.push((symbol - rle_max as usize) as u8);
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

fn read_block_count_code(reader: &mut BitReader, code: usize) -> Result<usize, BrotliError> {
    let (base, num_extra_bits) = match code {
        0 => (1, 2),
        1 => (5, 2),
        2 => (9, 2),
        3 => (13, 2),
        4 => (17, 3),
        5 => (25, 3),
        6 => (33, 3),
        7 => (41, 3),
        8 => (49, 4),
        9 => (65, 4),
        10 => (81, 4),
        11 => (97, 4),
        12 => (113, 5),
        13 => (145, 5),
        14 => (177, 5),
        15 => (209, 5),
        16 => (241, 6),
        17 => (305, 6),
        18 => (369, 7),
        19 => (497, 8),
        20 => (753, 9),
        21 => (1265, 10),
        22 => (2289, 11),
        23 => (4337, 12),
        24 => (8433, 13),
        25 => (16625, 24),
        _ => unreachable!("Invalid block count code: {code}"),
    };
    let extra_bits = reader
        .read_bits::<usize>(num_extra_bits)
        .map_err(BrotliError::from)?;

    Ok(base + extra_bits)
}
fn read_insert_length_code(reader: &mut BitReader, code: usize) -> Result<usize, BrotliError> {
    let (base, num_extra_bits) = match code {
        0 => (0, 0),
        1 => (1, 0),
        2 => (2, 0),
        3 => (3, 0),
        4 => (4, 0),
        5 => (5, 0),
        6 => (6, 1),
        7 => (8, 1),
        8 => (10, 2),
        9 => (14, 2),
        10 => (18, 3),
        11 => (26, 3),
        12 => (34, 4),
        13 => (50, 4),
        14 => (66, 5),
        15 => (98, 5),
        16 => (130, 6),
        17 => (194, 7),
        18 => (322, 8),
        19 => (578, 9),
        20 => (1090, 10),
        21 => (2114, 12),
        22 => (6210, 14),
        23 => (22594, 24),
        _ => unreachable!("Invalid insert length code: {code}"),
    };

    let extra_bits = reader
        .read_bits::<usize>(num_extra_bits)
        .map_err(BrotliError::from)?;

    Ok(base + extra_bits)
}

fn read_copy_length_code(reader: &mut BitReader, code: usize) -> Result<usize, BrotliError> {
    let (base, num_extra_bits) = match code {
        0 => (2, 0),
        1 => (3, 0),
        2 => (4, 0),
        3 => (5, 0),
        4 => (6, 0),
        5 => (7, 0),
        6 => (8, 0),
        7 => (9, 0),
        8 => (10, 1),
        9 => (12, 1),
        10 => (14, 2),
        11 => (18, 2),
        12 => (22, 3),
        13 => (30, 3),
        14 => (38, 4),
        15 => (54, 4),
        16 => (70, 5),
        17 => (102, 5),
        18 => (134, 6),
        19 => (198, 7),
        20 => (326, 8),
        21 => (582, 9),
        22 => (1094, 10),
        23 => (2118, 24),
        _ => unreachable!("Invalid copy length code: {code}"),
    };

    let extra_bits = reader
        .read_bits::<usize>(num_extra_bits)
        .map_err(BrotliError::from)?;

    Ok(base + extra_bits)
}

/// Read the block type metadata from the meta header
fn decode_blockdata(
    reader: &mut BitReader,
) -> Result<
    (
        usize,
        Option<HuffmanTree<Bits<usize>>>,
        Option<HuffmanTree<Bits<usize>>>,
        usize,
    ),
    BrotliError,
> {
    let num_blocks = decode_blocknum(reader)? as usize;

    if num_blocks >= 2 {
        let block_type_prefix_code = read_prefix_code(reader, num_blocks + 2)?;
        let block_count_prefix_code = read_prefix_code(reader, 26)?;
        let first_block_count_code = block_count_prefix_code
            .lookup_incrementally(reader)
            .map_err(|_| BrotliError::InvalidSymbol)?
            .ok_or(BrotliError::SymbolNotFound)?
            .val() as usize;
        let first_literal_block_count = read_block_count_code(reader, first_block_count_code)?;

        Ok((
            num_blocks,
            Some(block_type_prefix_code),
            Some(block_count_prefix_code),
            first_literal_block_count,
        ))
    } else {
        Ok((num_blocks, None, None, 16777216))
    }
}

fn decode_literal_context_id(context_mode: u8, last_two_bytes: &[u8; 2]) -> u8 {
    let p1 = last_two_bytes[1];
    let p2 = last_two_bytes[0];

    match context_mode {
        0 => {
            // LSB6 Mode
            p1 & 0x3f
        },
        1 => {
            // MSB6 Mode
            p1 >> 2
        },
        2 => {
            // UTF8 Mode
            LUT0[p1 as usize] | LUT1[p2 as usize]
        },
        3 => {
            // Signed Mode
            (LUT2[p1 as usize] << 3) | LUT2[p2 as usize]
        },
        _ => unreachable!("invalid context mode: {context_mode}"),
    }
}

fn decode_distance_context_id(copy_length: usize) -> usize {
    match copy_length {
        2 => 0,
        3 => 1,
        4 => 2,
        5.. => 3,
        _ => unreachable!("invalid copy length: {copy_length}"),
    }
}

fn distance_short_code_substitution(
    distance_code: usize,
    past_distances: &RingBuffer<usize>,
    npostfix: usize,
    ndirect: usize,
    reader: &mut BitReader,
) -> Result<usize, BrotliError> {
    let postfix_mask = (1 << npostfix) - 1;
    println!(
        "distance code {distance_code} last dist {:?}",
        past_distances
    );
    let distance = match distance_code {
        0 => *past_distances.nth_last(0),
        1 => *past_distances.nth_last(1),
        2 => *past_distances.nth_last(2),
        3 => *past_distances.nth_last(3),
        4 => *past_distances.nth_last(0) - 1,
        5 => *past_distances.nth_last(0) + 1,
        6 => *past_distances.nth_last(0) - 2,
        7 => *past_distances.nth_last(0) + 2,
        8 => *past_distances.nth_last(0) - 3,
        9 => *past_distances.nth_last(0) + 3,
        10 => *past_distances.nth_last(1) - 1,
        11 => *past_distances.nth_last(1) + 1,
        12 => *past_distances.nth_last(1) - 2,
        13 => *past_distances.nth_last(1) + 2,
        14 => *past_distances.nth_last(1) - 3,
        15 => *past_distances.nth_last(1) + 3,
        d @ 16.. => {
            if d < 16 + ndirect {
                d - 15
            } else {
                let num_extra_bits = 1 + ((d - ndirect - 16) >> (npostfix + 1));
                // println!("bit reader pos {} {}", reader.byte_ptr, reader.bit_ptr);
                let extra_bits = reader
                    .read_bits::<usize>(num_extra_bits as u8)
                    .map_err(BrotliError::from)?;
                // println!("extra bits for distance: {extra_bits} {num_extra_bits}");
                let hcode = (d - ndirect - 16) >> npostfix;
                let lcode = (d - ndirect - 16) & postfix_mask;
                let offset = ((2 + (hcode & 1)) << num_extra_bits) - 4;
                // println!("hcode {hcode} lcode {lcode} offset {offset}");
                ((offset + extra_bits) << npostfix) + lcode + ndirect + 1
            }
        },
        _ => unreachable!(), // Literally unreachable, we checked for all the values above. Rust isn't able to infer the usize value range yet.
    };

    Ok(distance)
}

fn decode_insert_and_copy_length_code(code: usize) -> (usize, usize) {
    let (insert_base, copy_base) = match code {
        0..=63 => (0, 0),
        64..=127 => (0, 8),
        128..=191 => (0, 0),
        192..=255 => (0, 8),
        256..=319 => (8, 0),
        320..=383 => (8, 8),
        384..=447 => (0, 16),
        448..=511 => (16, 0),
        576..=639 => (16, 8),
        640..=703 => (16, 16),
        _ => unreachable!("invalid insert and copy length code: {code}"),
    };

    let insert_length_extra = (code >> 3) & 0b111;
    let copy_length_extra = code & 0b111;

    (
        insert_base + insert_length_extra,
        copy_base + copy_length_extra,
    )
}

/// Parse the block length code given in a <block, switch> command
///
/// Returns a tuple of `(base, num_extra_bits)`
/// The final code is given by `base + read(num_extra_bits)`
fn decode_blocklen(blen_code: usize) -> (usize, usize) {
    match blen_code {
        0 => (1, 2),
        1 => (5, 2),
        2 => (9, 2),
        3 => (13, 2),
        4 => (17, 3),
        5 => (25, 3),
        6 => (33, 3),
        7 => (41, 3),
        8 => (49, 4),
        9 => (65, 4),
        10 => (81, 4),
        11 => (97, 4),
        12 => (113, 5),
        13 => (145, 5),
        14 => (177, 5),
        15 => (209, 5),
        16 => (241, 6),
        17 => (305, 6),
        18 => (369, 7),
        19 => (497, 8),
        20 => (753, 9),
        21 => (1265, 10),
        22 => (2289, 11),
        23 => (4337, 12),
        24 => (8443, 13),
        25 => (16625, 24),
        _ => unreachable!("invalid block length code {blen_code}"),
    }
}
