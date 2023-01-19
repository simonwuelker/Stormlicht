//! [zlib](https://www.rfc-editor.org/rfc/rfc1950) implementation
//!
//! ZLIB is basically just a thin wrapper around deflate.

use anyhow::{Context, Result};
use thiserror::Error;

use crate::deflate;

#[derive(Debug, Error)]
pub enum ZLibError {
    #[error("Unexpected end of file")]
    UnexpectedEOF,
    #[error("Reserved compression method 15")]
    ReservedCompressionMethod,
    #[error("CINFO too large, must be smaller or equal to 7, found {}", .0)]
    CINFOTooLarge(u8),
    #[error("Invalid zlib header checksum, found {}, must be multiple of 31", .0)]
    InvalidHeaderChecksum(u16),
    #[error("Unknown compression method: {}", .0)]
    UnknownCompressionMethod(u8),
    #[error("Incorrect zlib data checksum, expected {expected}, found {found}, this indicates a bug in the deflate implementation")]
    IncorrectDataChecksum { expected: u32, found: u32 },
}

/// Minimum length for a zlib archive
///
/// Consists of:
/// * Compression method (1 byte)
/// * Compression flags (1 byte)
/// * Adler32 checksum (4 bytes)
///
/// Note that the minimum length of a DEFLATE archive is not included since zlib may use algorithms other than DEFLATE.
const MINIMUM_ZLIB_LEN: usize = 6;

pub fn decode(bytes: &[u8]) -> Result<Vec<u8>> {
    if bytes.len() < MINIMUM_ZLIB_LEN {
        return Err(ZLibError::UnexpectedEOF.into());
    }

    // parse CMF
    let compression_method_and_flags = bytes[0];
    let compression_method = compression_method_and_flags & 0b1111;
    let compression_info = compression_method_and_flags >> 4;

    // Parse FLG
    let flags = bytes[2];
    let flag_dict = ((flags & 1) << 5) != 0;
    let _flag_level = flags >> 6; // compression level, not needed for decompression

    let header_checksum = u16::from_be_bytes(bytes[..2].try_into().unwrap());
    if header_checksum % 31 != 0 {
        return Err(ZLibError::InvalidHeaderChecksum(header_checksum).into());
    }

    match compression_method {
        8 => {
            // DEFLATE
            if 7 < compression_info {
                return Err(ZLibError::CINFOTooLarge(compression_info).into());
            }

            let _lz77_window_size = 1 << (compression_info as usize + 8);

            if flag_dict {
                todo!("Implement flag_dict flag");
            }

            let (decompressed, num_consumed_bytes) =
                deflate::decode(&bytes[2..]).context("Failed to decompress zlib body")?;

            let expected_checksum =
                u32::from_be_bytes(bytes[2 + num_consumed_bytes..][..4].try_into().unwrap());
            let computed_checksum = adler32(&decompressed);

            if expected_checksum != computed_checksum {
                return Err(ZLibError::IncorrectDataChecksum {
                    expected: expected_checksum,
                    found: computed_checksum,
                }
                .into());
            }

            Ok(decompressed)
        },
        15 => Err(ZLibError::ReservedCompressionMethod.into()),
        _ => Err(ZLibError::UnknownCompressionMethod(compression_method).into()),
    }
}

pub fn adler32(bytes: &[u8]) -> u32 {
    let mut s1: u32 = 1;
    let mut s2: u32 = 0;
    for byte in bytes {
        s1 = (s1 + *byte as u32) % 65521;
        s2 = (s2 + s1) % 65521;
    }

    s2 << 16 | s1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adler32() {
        let bytes = b"Hello World";
        assert_eq!(adler32(bytes), 403375133)
    }

    #[test]
    fn test_zlib_decompression() -> Result<()> {
        let bytes = [
            0x78, 0x9c, 0x4b, 0x4c, 0x4a, 0x06, 0x00, 0x02, 0x4d, 0x01, 0x27,
        ];
        let decompressed = decode(&bytes)?;

        assert_eq!(&decompressed, b"abc");
        Ok(())
    }
}
