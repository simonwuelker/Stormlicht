//! [zlib](https://www.rfc-editor.org/rfc/rfc1950) implementation
//!
//! ZLIB is basically just a thin wrapper around deflate.

use hash::Adler32;

use crate::deflate;

#[derive(Clone, Copy, Debug)]
pub enum ZLibError {
    UnexpectedEOF,
    /// Usage of the reserved compression method `15`.
    ReservedCompressionMethod,
    /// `CINFO` must be smaller or equal to 7
    CINFOTooLarge,
    /// ZLIB Header Checksum must be a multiple of 31
    InvalidHeaderChecksum,
    /// An error occured during the `DEFLATE` decompression
    Deflate(deflate::DeflateError),
    UnknownCompressionMethod,
    /// The checksum of the decompressed data was incorrect
    IncorrectDataChecksum,
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

pub fn decode(bytes: &[u8]) -> Result<Vec<u8>, ZLibError> {
    if bytes.len() < MINIMUM_ZLIB_LEN {
        return Err(ZLibError::UnexpectedEOF);
    }

    // parse Compression method and flags (CMF)
    let compression_method_and_flags = bytes[0];
    let compression_method = compression_method_and_flags & 0b1111;
    let compression_info = compression_method_and_flags >> 4;

    // Parse compression flags (FLG)
    let flags = bytes[1];
    let flag_dict = ((flags & 1) << 5) != 0;
    let _flag_level = flags >> 6; // compression level, not needed for decompression

    let header_checksum = u16::from_be_bytes(bytes[..2].try_into().unwrap());
    if header_checksum % 31 != 0 {
        log::warn!("Invalid zlib header checksum: {header_checksum} (must be a multiple of 31)");
        return Err(ZLibError::InvalidHeaderChecksum);
    }

    match compression_method {
        8 => {
            // DEFLATE
            if 7 < compression_info {
                return Err(ZLibError::CINFOTooLarge);
            }

            let _lz77_window_size = 1 << (compression_info as usize + 8);

            if flag_dict {
                todo!("Implement flag_dict flag");
            }

            let (decompressed, num_consumed_bytes) =
                deflate::decode(&bytes[2..]).map_err(ZLibError::Deflate)?;

            // Verify the checksum provided after the compressed data
            let expected_checksum =
                u32::from_be_bytes(bytes[2 + num_consumed_bytes..][..4].try_into().unwrap());

            let mut hasher = Adler32::default();
            hasher.write(&decompressed);
            let computed_checksum = hasher.finish();

            if expected_checksum != computed_checksum {
                log::warn!("Incorrect zlib checksum: expected {expected_checksum:0>8x}, found {computed_checksum:0>8x}");
                return Err(ZLibError::IncorrectDataChecksum);
            }

            Ok(decompressed)
        },
        15 => {
            log::warn!("Reserved zlib compression method");
            Err(ZLibError::ReservedCompressionMethod)
        },
        _ => {
            log::warn!("Unknown zlib compression method: {compression_method}");
            Err(ZLibError::UnknownCompressionMethod)
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zlib_decompression() -> Result<(), ZLibError> {
        let bytes = [
            0x78, 0x9c, 0x4b, 0x4c, 0x4a, 0x06, 0x00, 0x02, 0x4d, 0x01, 0x27,
        ];
        let decompressed = decode(&bytes)?;

        assert_eq!(&decompressed, b"abc");
        Ok(())
    }
}
