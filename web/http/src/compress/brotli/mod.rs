//! Implements the [Brotli](https://datatracker.ietf.org/doc/html/rfc7932) compression algorithm

pub mod huffman;

use crate::compress::bit_reader::{BitReader, BitReaderError};

use self::huffman::{HuffmanTree, Bits, Code};

#[derive(Debug)]
pub enum BrotliError {
	UnexpectedEOF,
	InvalidFormat,
	InvalidSymbol,
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

		println!("mnibbles: {}", mnibbles);

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
			reader.read_bits::<u32>(4 * mnibbles).map_err(BrotliError::from)? as usize + 1
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

		println!("npostfix {npostfix} ndirect {ndirect}");

		let mut context_modes_for_literal_block_types = Vec::with_capacity(num_blocks[NBLTYPESL] as usize);
		for _ in 0..num_blocks[NBLTYPESL] {
			let context_mode = reader.read_bits::<u8>(2).map_err(BrotliError::from)?;
			context_modes_for_literal_block_types.push(context_mode);
		}

		// read NTREES
		let ntreesl = decode_blocknum(&mut reader)?;
		if ntreesl >= 2 {
			// parse context map literals
			decode_context_map(&mut reader, ntreesl, 64 * num_blocks[NBLTYPESL] as usize)?;
		} else {
			// fill cmapl with zeros
		}

		let ntreesd = decode_blocknum(&mut reader)?;
		if ntreesd >= 2 {
			// decode_context_map(&mut reader, ntreesd, 4 * num_blocks[NBLTYPESD] as usize)?;
			todo!();
		} else {
			// fill cmapd with zeros
		}

		println!("trees {ntreesl} {ntreesd}");

		// read array of literal prefix codes, HTREEL[]




	}
	Ok(result)
}

fn read_prefix_code(reader: &mut BitReader, alphabet_size: usize) -> Result<HuffmanTree<Bits<u32>>, BrotliError> {
	let alphabet_width = 16 - (alphabet_size as u16 - 1).leading_zeros() as u8;

	let ident = reader.read_bits::<u8>(2).map_err(BrotliError::from)?;
	let mut symbols_raw = vec![];

	let huffmantree = if ident == 1 {
		// Simple prefix code
		let nsym = reader.read_bits::<u8>(2).map_err(BrotliError::from)? + 1;

		// read nsym symbols
		for _ in 0..nsym {
			let symbol_raw = reader.read_bits::<u32>(alphabet_width).map_err(BrotliError::from)?;
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
		let symbols = symbols_raw.into_iter()
			.map(|raw_symbol|Bits::new(raw_symbol, alphabet_width as usize))
			.collect();

		HuffmanTree::new_infer_codes(symbols, lengths)
	} else {
		// Complex prefix code
		todo!("complex prefix codes")
	};
	Ok(huffmantree)
}

fn decode_blocknum(reader: &mut BitReader) -> Result<u8, BrotliError> {
	if reader.read_single_bit().map_err(BrotliError::from)? {
		let num_extrabits = reader.read_bits::<u8>(3).map_err(BrotliError::from)?;

		if num_extrabits > 7 {
			return Err(BrotliError::InvalidFormat);
		}

		let extra = reader.read_bits::<u8>(num_extrabits).map_err(BrotliError::from)?;
		Ok((1 << num_extrabits) + 1 + extra)
	} else {
		Ok(1)
	}
}

/// https://www.rfc-editor.org/rfc/rfc7932#section-7.3
fn decode_context_map(reader: &mut BitReader, num_trees: u8, size: usize) -> Result<Vec<u8>, BrotliError> {
	let rle_max = match reader.read_single_bit().map_err(BrotliError::from)? {
		false => 0,
		true => {
			reader.read_bits::<u8>(4).map_err(BrotliError::from)? + 1
		}
	};

	let prefix_code = read_prefix_code(reader, (num_trees + rle_max) as usize)?;

	let mut context_map = Vec::with_capacity(size);

	Ok(context_map)
}