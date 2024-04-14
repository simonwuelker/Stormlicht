use crate::huffman::HuffmanTree;

use sl_std::bitreader::{self, BitReader};

use std::cmp::{min, Ordering};

#[derive(Clone, Copy, Debug)]
pub enum Error {
    InvalidCompressionScheme,
    ReservedCompressionScheme,
    UnexpectedEOF,
    SymbolNotFound,
    RLELeadingRepeatValue,
    RLEExceedsExpectedLength,
    InvalidUncompressedBlockLength,
    BitReader(bitreader::Error),
}

impl From<bitreader::Error> for Error {
    fn from(value: bitreader::Error) -> Self {
        Self::BitReader(value)
    }
}

#[derive(Clone, Copy, Debug)]
enum CompressionScheme {
    Uncompressed,
    FixedHuffmanCodes,
    DynamicHuffmanCodes,
    Reserved,
}

/// Returns a tuple of `(decompressed_bytes, num_consumed_compressed_bytes)` on success
pub fn decompress(source: &[u8]) -> Result<(Vec<u8>, usize), Error> {
    let mut reader = BitReader::new(source);
    let mut output_stream = vec![];

    let mut default_lit_lenghts = vec![8; 144];
    default_lit_lenghts.extend(vec![9; 112]);
    default_lit_lenghts.extend(vec![7; 24]);
    default_lit_lenghts.extend(vec![8; 8]);

    let default_lit_tree = HuffmanTree::new_infer_codes_without_symbols(&default_lit_lenghts);
    let default_dist_tree = HuffmanTree::new_infer_codes_without_symbols(&[5; 32]);

    loop {
        let is_final = reader.read_single_bit()?;
        let btype = reader.read_bits::<u8>(2)?.try_into()?;
        match btype {
            CompressionScheme::Uncompressed => {
                reader.align_to_byte_boundary();
                let len = reader.read_bits::<u16>(16)?;
                let nlen = reader.read_bits::<u16>(16)?;

                if len ^ 0xFFFF != nlen {
                    return Err(Error::InvalidUncompressedBlockLength);
                }

                output_stream.reserve(len as usize);

                for _ in 0..len {
                    output_stream.push(reader.read_bits::<u8>(8)?);
                }
            },
            CompressionScheme::DynamicHuffmanCodes => {
                // Read the huffman codes from the start of the block
                let hlit = reader.read_bits::<usize>(5)? + 257;
                let hdist = reader.read_bits::<usize>(5)? + 1;
                let hclen = reader.read_bits::<usize>(4)? + 4;

                let (literal_tree, distance_tree) =
                    read_literal_and_distance_tree(hlit, hdist, hclen, &mut reader)?;
                decompress_block(
                    &literal_tree,
                    &distance_tree,
                    &mut reader,
                    &mut output_stream,
                )?;
            },
            CompressionScheme::FixedHuffmanCodes => {
                decompress_block(
                    &default_lit_tree,
                    &default_dist_tree,
                    &mut reader,
                    &mut output_stream,
                )?;
            },
            CompressionScheme::Reserved => {
                return Err(Error::ReservedCompressionScheme);
            },
        }

        if is_final {
            break;
        }
    }
    Ok((output_stream, reader.num_consumed_bytes()))
}

fn decompress_block(
    literal_tree: &HuffmanTree<usize>,
    distance_tree: &HuffmanTree<usize>,
    reader: &mut BitReader<'_>,
    output_stream: &mut Vec<u8>,
) -> Result<(), Error> {
    'decompress_block: loop {
        let symbol = *literal_tree
            .lookup_incrementally(reader)
            .map_err(|_| Error::UnexpectedEOF)?
            .ok_or(Error::SymbolNotFound)?;

        match symbol.cmp(&256) {
            Ordering::Less => output_stream.push(symbol as u8),
            Ordering::Equal => break 'decompress_block,
            Ordering::Greater => {
                let run_length = decode_run_length(symbol, reader)?;
                let distance_code = *distance_tree
                    .lookup_incrementally(reader)
                    .map_err(|_| Error::UnexpectedEOF)?
                    .ok_or(Error::SymbolNotFound)?;
                let distance = decode_distance(distance_code, reader)?;

                let copy_base = output_stream.len() - distance;

                // TODO this, and probably most of the implemenentation, should be unifiied with compression::brotli
                let mut bytes_remaining = run_length;
                let bytes_to_copy_at_once = min(run_length, output_stream.len() - copy_base);

                while bytes_remaining > bytes_to_copy_at_once {
                    output_stream.extend_from_within(copy_base..copy_base + bytes_to_copy_at_once);
                    bytes_remaining -= bytes_to_copy_at_once;
                }

                output_stream.extend_from_within(copy_base..copy_base + bytes_remaining);
            },
        }
    }
    Ok(())
}

const CODE_LENGTH_ALPHABET: [usize; 19] = [
    16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
];

fn read_literal_and_distance_tree(
    hlit: usize,
    hdist: usize,
    hclen: usize,
    reader: &mut BitReader<'_>,
) -> Result<(HuffmanTree<usize>, HuffmanTree<usize>), Error> {
    let mut code_lengths = vec![0; 19];

    for index in &CODE_LENGTH_ALPHABET[..hclen] {
        code_lengths[*index] = reader.read_bits::<usize>(3)?;
    }

    let code_tree = HuffmanTree::new_infer_codes_without_symbols(&code_lengths);

    let total_number_of_codes = hlit + hdist;
    let mut codes: Vec<usize> = Vec::with_capacity(total_number_of_codes);
    while codes.len() < total_number_of_codes {
        let symbol = *code_tree
            .lookup_incrementally(reader)
            .map_err(|_| Error::UnexpectedEOF)?
            .ok_or(Error::SymbolNotFound)?;

        match symbol {
            0..=15 => {
                codes.push(symbol);
            },
            16 => {
                let to_repeat = *codes.last().ok_or(Error::RLELeadingRepeatValue)?;
                let repeat_for = reader.read_bits::<usize>(2)? + 3;

                if total_number_of_codes < codes.len() + repeat_for {
                    return Err(Error::RLEExceedsExpectedLength);
                }

                codes.resize(codes.len() + repeat_for, to_repeat);
            },
            17 => {
                let repeat_for = reader.read_bits::<usize>(3)? + 3;

                if total_number_of_codes < codes.len() + repeat_for {
                    return Err(Error::RLEExceedsExpectedLength);
                }

                codes.resize(codes.len() + repeat_for, 0);
            },
            18 => {
                let repeat_for = reader.read_bits::<usize>(7)? + 11;

                if total_number_of_codes < codes.len() + repeat_for {
                    return Err(Error::RLEExceedsExpectedLength);
                }

                codes.resize(codes.len() + repeat_for, 0);
            },
            _ => unreachable!("Invalid run length encoding value: {symbol}"),
        }
    }

    let literal_codes = &codes[..hlit];
    let distance_codes = &codes[hlit..];

    let literal_tree = HuffmanTree::new_infer_codes_without_symbols(literal_codes);
    let dist_tree = HuffmanTree::new_infer_codes_without_symbols(distance_codes);
    Ok((literal_tree, dist_tree))
}

fn decode_distance(code: usize, reader: &mut BitReader<'_>) -> Result<usize, Error> {
    let (base, num_extra_bits) = match code {
        0 => (1, 0),
        1 => (2, 0),
        2 => (3, 0),
        3 => (4, 0),
        4 => (5, 1),
        5 => (7, 1),
        6 => (9, 2),
        7 => (13, 2),
        8 => (17, 3),
        9 => (25, 3),
        10 => (33, 4),
        11 => (49, 4),
        12 => (65, 5),
        13 => (97, 5),
        14 => (129, 6),
        15 => (193, 6),
        16 => (257, 7),
        17 => (385, 7),
        18 => (513, 8),
        19 => (769, 8),
        20 => (1025, 9),
        21 => (1537, 9),
        22 => (2049, 10),
        23 => (3073, 10),
        24 => (4097, 11),
        25 => (6145, 11),
        26 => (8193, 12),
        27 => (12289, 12),
        28 => (16385, 13),
        29 => (24577, 13),
        _ => unreachable!("Invalid distance code: {code}"),
    };
    let extra_bits = reader.read_bits::<usize>(num_extra_bits)?;
    Ok(base + extra_bits)
}

fn decode_run_length(code: usize, reader: &mut BitReader<'_>) -> Result<usize, Error> {
    let (base, num_extra_bits) = match code {
        257 => (3, 0),
        258 => (4, 0),
        259 => (5, 0),
        260 => (6, 0),
        261 => (7, 0),
        262 => (8, 0),
        263 => (9, 0),
        264 => (10, 0),
        265 => (11, 1),
        266 => (13, 1),
        267 => (15, 1),
        268 => (17, 1),
        269 => (19, 2),
        270 => (23, 2),
        271 => (27, 2),
        272 => (31, 2),
        273 => (35, 3),
        274 => (43, 3),
        275 => (51, 3),
        276 => (59, 3),
        277 => (67, 4),
        278 => (83, 4),
        279 => (99, 4),
        280 => (115, 4),
        281 => (131, 5),
        282 => (163, 5),
        283 => (195, 5),
        284 => (227, 5),
        285 => (258, 0),
        _ => unreachable!("Invalid distance code: {code}"),
    };

    let extra_bits = reader.read_bits::<usize>(num_extra_bits)?;
    Ok(base + extra_bits)
}

impl TryFrom<u8> for CompressionScheme {
    type Error = Error;

    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Uncompressed),
            1 => Ok(Self::FixedHuffmanCodes),
            2 => Ok(Self::DynamicHuffmanCodes),
            3 => Ok(Self::Reserved),
            _ => {
                log::warn!("Invalid DEFLATE compression scheme: {value}");
                Err(Error::InvalidCompressionScheme)
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{decompress, Error};

    #[test]
    fn test_basic() -> Result<(), Error> {
        let bytes = [0x4b, 0x4c, 0x4a, 0x06, 0x00];
        let (decompressed, num_consumed_bytes) = decompress(&bytes)?;

        assert_eq!(&decompressed, b"abc");
        assert_eq!(num_consumed_bytes, bytes.len());
        Ok(())
    }
}
