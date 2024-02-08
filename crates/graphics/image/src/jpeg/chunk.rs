use std::iter::FusedIterator;

use super::Error;

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub(super) enum Chunk<'a> {
    /// `SOI`
    StartOfImage,

    /// `SOF`
    StartOfFrame { subscript: u8, data: &'a [u8] },

    /// `DHT`
    DefineHuffmanTable(&'a [u8]),

    /// `DQT`
    DefineQuantizationTable(&'a [u8]),

    /// `DRI`
    DefineRestartInterval(&'a [u8]),

    /// `SOS`
    StartOfScan { data: &'a [u8], scan: Vec<u8> },

    /// `RST`
    Restart(u8),

    /// `APP`
    ApplicationSpecific { subscript: u8, data: &'a [u8] },

    /// `COM`
    Comment(&'a [u8]),

    /// `EOI`
    EndOfImage,
}

impl<'a> Chunk<'a> {
    const SOI: u8 = 0xD8;
    const SOF_BEGIN: u8 = 0xC0;
    const SOF_END: u8 = 0xC3;
    const DHT: u8 = 0xC4;
    const DQT: u8 = 0xDB;
    const DRI: u8 = 0xDD;
    const SOS: u8 = 0xDA;
    const RST_BEGIN: u8 = 0xD0;
    const RST_END: u8 = 0xD7;
    const APP_BEGIN: u8 = 0xE0;
    const APP_END: u8 = 0xEF;
    const COM: u8 = 0xFE;
    const EOI: u8 = 0xD9;

    /// Tries to read a chunk from the byte slice
    ///
    /// On success this returns the chunk and the remaining bytes
    pub fn read(bytes: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if bytes.len() < 2 || bytes[0] != 0xFF {
            return Err(Error::BadChunk);
        }

        let chunk_id = bytes[1];
        let (chunk, length) = match chunk_id {
            Self::SOI => (Self::StartOfImage, 0),
            n @ (Self::SOF_BEGIN..=Self::SOF_END) => {
                let data = read_variable_sized_chunk(&bytes[2..])?;
                let subscript = n - Self::SOF_BEGIN;
                let chunk = Self::StartOfFrame { subscript, data };
                (chunk, data.len() + 2)
            },
            Self::DHT => {
                let chunk_data = read_variable_sized_chunk(&bytes[2..])?;
                (Self::DefineHuffmanTable(chunk_data), chunk_data.len() + 2)
            },
            Self::DQT => {
                let chunk_data = read_variable_sized_chunk(&bytes[2..])?;
                (
                    Self::DefineQuantizationTable(chunk_data),
                    chunk_data.len() + 2,
                )
            },
            Self::DRI => {
                let chunk_data = read_variable_sized_chunk(&bytes[2..])?;
                (
                    Self::DefineRestartInterval(chunk_data),
                    chunk_data.len() + 2,
                )
            },
            Self::SOS => {
                let data = read_variable_sized_chunk(&bytes[2..])?;

                let remaining_bytes = &bytes[data.len() + 4..];
                let (scan, scan_length) = read_compressed_data(remaining_bytes)?;
                let chunk = Self::StartOfScan { data, scan };
                (chunk, data.len() + scan_length + 2)
            },
            n @ (Self::RST_BEGIN..=Self::RST_END) => (Self::Restart(n - Self::RST_BEGIN), 0),
            n @ (Self::APP_BEGIN..=Self::APP_END) => {
                let data = read_variable_sized_chunk(&bytes[2..])?;
                let subscript = n - Self::APP_BEGIN;
                let app_chunk = Self::ApplicationSpecific { subscript, data };
                (app_chunk, data.len() + 2)
            },
            Self::COM => {
                let chunk_data = read_variable_sized_chunk(&bytes[2..])?;
                (Self::Comment(chunk_data), chunk_data.len() + 2)
            },
            Self::EOI => (Self::EndOfImage, 0),
            other => {
                log::error!("Unknown jpeg chunk: {other:?}");
                return Err(Error::UnknownChunk);
            },
        };

        let remaining_bytes = &bytes[length + 2..];
        Ok((chunk, remaining_bytes))
    }
}

fn read_variable_sized_chunk(bytes: &[u8]) -> Result<&[u8], Error> {
    if bytes.len() < 2 {
        return Err(Error::BadChunk);
    }

    // The length of the chunk includes the length bytes themselves
    let length = u16::from_be_bytes(
        bytes[..2]
            .try_into()
            .expect("Slice is exactly 2 elements long"),
    ) as usize;

    if bytes.len() < length {
        return Err(Error::BadChunk);
    }

    let chunk_data = &bytes[2..length];

    Ok(chunk_data)
}

#[derive(Clone, Copy)]
pub(super) struct Chunks<'a> {
    remaining_bytes: &'a [u8],
}

impl<'a> Chunks<'a> {
    #[must_use]
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            remaining_bytes: bytes,
        }
    }
}

impl<'a> Iterator for Chunks<'a> {
    type Item = Result<Chunk<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining_bytes.is_empty() {
            return None;
        }

        match Chunk::read(self.remaining_bytes) {
            Ok((chunk, remaining)) => {
                self.remaining_bytes = remaining;
                Some(Ok(chunk))
            },
            Err(e) => Some(Err(e)),
        }
    }
}

impl<'a> FusedIterator for Chunks<'a> {}

/// Extracts the compressed data following a `SOS` chunk
///
/// Returns the unescaped compressed data as well as the length (in the escaped form)
fn read_compressed_data(bytes: &[u8]) -> Result<(Vec<u8>, usize), Error> {
    // The compressed data ends once we find a chunk marker that is not
    // * FFD0 - FFD7 (Restart markers)
    // * FF00 (escaped 0xFF byte)

    let mut elements = bytes.iter().enumerate();
    let mut compressed_data = vec![];
    let mut pushed_until = 0;

    while let Some((index, &element)) = elements.next() {
        if element == 0xFF {
            let Some((next_index, &next_element)) = elements.next() else {
                // This is a 0xFF at the very end of the data stream.
                // This should never happen with a compliant encoder
                return Err(Error::BadChunk);
            };

            if matches!(next_element, Chunk::RST_BEGIN..=Chunk::RST_END) {
                // This is a restart marker, leave it as-is
            } else if next_element == 0x00 {
                // This is a escaped 0xFF byte
                compressed_data.extend(&bytes[pushed_until..index]);
                pushed_until = next_index + 1;
            } else {
                // This is a marker after the end of the compressed data.
                compressed_data.extend(&bytes[pushed_until..index]);
                return Ok((compressed_data, index));
            }
        }
    }

    // We have reached the end of the byte stream, this means that there is nothing
    // after the compressed data.
    // This is impossible, as there must be a EOI at the end of the image
    Err(Error::BadChunk)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_compressed_data_after_sos() {
        // Immediate EOI - there is no compressed data
        let empty = &[0xFF, 0xD9];
        assert!(read_compressed_data(empty).unwrap().0.is_empty());

        // Data with reset markers inbetween
        let with_rst = &[0xAA, 0xFF, 0xD0, 0xAA, 0xFF, 0xD7, 0xAA, 0xFF, 0xD9];
        assert_eq!(
            &read_compressed_data(with_rst).unwrap().0,
            &[0xAA, 0xFF, 0xD0, 0xAA, 0xFF, 0xD7, 0xAA]
        );

        // Data with escaped 0xFF bytes
        let with_escape = &[0xFF, 0x00, 0xAA, 0xFF, 0xD9];
        assert_eq!(&read_compressed_data(with_escape).unwrap().0, &[0xAA]);
    }
}
