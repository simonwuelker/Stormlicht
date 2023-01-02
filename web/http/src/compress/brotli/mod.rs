//! Implements the [Brotli](https://datatracker.ietf.org/doc/html/rfc7932) compression algorithm

use std::io::Read;
use crate::compress::bit_reader::{BitReader, BitReaderError};

#[derive(Debug)]
pub enum BrotliError {
	UnexpectedEOF,
	InvalidFormat,
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
		is_last = reader.read_bits::<u8>(1).map_err(BrotliError::from)? == 1;

		if is_last {
			// read ISLASTEMPTY bit
			if reader.read_bits::<u8>(1).map_err(BrotliError::from)? == 1 {
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
			if reader.read_bits::<u8>(1).map_err(BrotliError::from)? != 0 {
				return Err(BrotliError::InvalidFormat);
			}

			// read MSKIPLEN
			todo!("do empty blocks even occur in the real word");
			// skip any bits up to the next byte boundary
		} else {
			// read MLEN
			reader.read_bits::<u32>(4 * mnibbles).map_err(BrotliError::from)? as usize
		};

		println!("mlen: {mlen}");

		if !is_last {
			let is_uncompressed = reader.read_bits::<u8>(1).map_err(BrotliError::from)? == 1;

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
		let ndirect = reader.read_bits::<u8>(4).map_err(BrotliError::from)? << npostfix;

		let mut context_modes_for_literal_block_types = Vec::with_capacity(num_blocks[NBLTYPESL] as usize);

		for _ in 0..num_blocks[NBLTYPESL] {
			let context_mode = reader.read_bits::<u8>(2).map_err(BrotliError::from)?;
			context_modes_for_literal_block_types.push(context_mode);
		}

		// read NTREES
		let ntreesl = decode_blocknum(&mut reader)?;
		if ntreesl > 2 {
			todo!();
		}

		let ntreesd = decode_blocknum(&mut reader)?;
		if ntreesd > 2 {
			todo!()
		}




	}
	Ok(result)
}

fn read_prefix_code(reader: &mut BitReader, alphabet_bits: u8) -> Result<(), BrotliError> {
	let ident = reader.read_bits::<u8>(2).map_err(BrotliError::from)?;

	if ident == 1 {
		// Simple prefix code
		let nsym = reader.read_bits::<u8>(2).map_err(BrotliError::from)? + 1;

	} else {
		// Complex prefix code
		todo!()
	}
	Ok(())
}

fn decode_blocknum(reader: &mut BitReader) -> Result<u8, BrotliError> {
	if reader.read_bits::<u8>(1).map_err(BrotliError::from)? == 1 {
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