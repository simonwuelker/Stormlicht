//! <https://www.rfc-editor.org/rfc/rfc1952>

use std::io::{self, Seek};

use sl_std::read::ReadExt;

use crate::deflate;

const GZIP_MAGIC: u16 = 0x8B1F;

mod flags {
    /// Flag indicating that a CRC-16 checksum is present
    pub const FHCRC: u8 = 1 << 1;

    /// Flag indicating whether extra fields are present
    pub const FEXTRA: u8 = 1 << 2;

    /// Flag indicating whether the original filename is present
    pub const FNAME: u8 = 1 << 3;

    /// Flag indicating whether a comment is present
    pub const FCOMMENT: u8 = 1 << 4;
}

#[derive(Clone, Copy, Debug)]
pub enum Error {
    NotGzip,
    UnknownCompressionMethod,
    UnexpectedEOF,
    UnexpectedLength,
    ChecksumError,
    Deflate(deflate::Error),
}

pub fn decompress(source_bytes: &[u8]) -> Result<Vec<u8>, Error> {
    let mut reader = io::Cursor::new(source_bytes);

    // Read the two ID bytes
    if reader.read_le_u16()? != GZIP_MAGIC {
        return Err(Error::NotGzip);
    }

    // At the time of writing, only compression method 8 (DEFLATE) is
    // standardized
    let compression_method = reader.read_le_u8()?;
    if compression_method != 8 {
        log::error!("Unknown compression method: {compression_method}");
        return Err(Error::UnknownCompressionMethod);
    }

    let flags = reader.read_be_u8()?;

    // Skip to the end of the header
    // This skips the MTIME, XFL and OS fields, as we do
    // not care about those
    reader.seek(io::SeekFrom::Start(10))?;

    if flags & flags::FEXTRA != 0 {
        let extra_length = reader.read_le_u16()?.into();

        // Skip the entire extra field
        reader.seek(io::SeekFrom::Current(extra_length))?;
    }

    if flags & flags::FNAME != 0 {
        skip_c_style_string(&mut reader)?;
    }

    if flags & flags::FCOMMENT != 0 {
        skip_c_style_string(&mut reader)?;
    }

    if flags & flags::FHCRC != 0 {
        // This checksum is for the header only
        // Since we don't care about *any* dynamic data
        // inside the header, we don't even verify it
        let _crc16 = reader.read_le_u16()?;
    }

    // After the header it becomes easier to not use a reader anymore
    let remaining = reader.remaining_slice();

    if remaining.len() < 8 {
        return Err(Error::UnexpectedEOF);
    }

    let deflate_bytes = &remaining[..remaining.len() - 8];
    let expected_crc32 = u32::from_le_bytes(
        remaining[remaining.len() - 8..remaining.len() - 4]
            .try_into()
            .expect("we checked the length before"),
    );
    let expected_length = u32::from_le_bytes(
        remaining[remaining.len() - 4..]
            .try_into()
            .expect("we checked the length before"),
    );

    let decompressed_bytes = deflate::decompress(deflate_bytes)?.0;

    // Note: The decompressed length is intentionally truncated (it is compared mod 2^32)
    if decompressed_bytes.len() as u32 != expected_length {
        log::error!(
            "Unexpected length of decompressed data: Expected {expected_length} bytes, got {}",
            decompressed_bytes.len()
        );
        return Err(Error::UnexpectedLength);
    }

    let computed_checksum = hash::crc32(&decompressed_bytes);
    if computed_checksum != expected_crc32 {
        log::error!("Checksum doesn't match: expected 0x{expected_crc32:8>0x}, found 0x{computed_checksum:8>0x}");
        return Err(Error::ChecksumError);
    }

    Ok(decompressed_bytes)
}

fn skip_c_style_string<R: io::Read>(mut reader: R) -> Result<(), io::Error> {
    while reader.read_le_u8()? != 0 {}
    Ok(())
}

impl From<io::Error> for Error {
    fn from(_value: io::Error) -> Self {
        // We only use a Cursor here, which can never fail except for eof
        Self::UnexpectedEOF
    }
}

impl From<deflate::Error> for Error {
    fn from(value: deflate::Error) -> Self {
        Self::Deflate(value)
    }
}
