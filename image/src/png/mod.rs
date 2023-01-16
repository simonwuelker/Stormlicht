//! Implements a [PNG](https://www.w3.org/TR/png) decoder

// The chunk types don't necessarily start with uppercase characters and renaming them would be silly
#![allow(non_upper_case_globals)]

pub mod chunks;

use anyhow::{Context, Result};
use std::fs;
use std::io::{Cursor, Read};
use std::path::Path;
use thiserror::Error;

use crate::crc32::IncrementalCRC32;

const PNG_HEADER: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

const IHDR: [u8; 4] = [73, 72, 68, 82];
const PLTE: [u8; 4] = [80, 76, 84, 69];
const IDAT: [u8; 4] = [73, 68, 65, 84];
const IEND: [u8; 4] = [73, 69, 78, 68];
const cHRM: [u8; 4] = [99, 72, 82, 77];
const dSIG: [u8; 4] = [100, 83, 73, 71];
const eXIF: [u8; 4] = [101, 88, 73, 102];
const gAMA: [u8; 4] = [103, 65, 77, 65];
const hIST: [u8; 4] = [104, 73, 83, 84];
const iCCP: [u8; 4] = [105, 67, 67, 80];
const iTXt: [u8; 4] = [105, 84, 88, 116];
const pHYs: [u8; 4] = [112, 72, 89, 115];
const sBIT: [u8; 4] = [115, 66, 73, 84];
const sPLT: [u8; 4] = [115, 80, 76, 84];
const sRGB: [u8; 4] = [115, 82, 71, 66];
const sTER: [u8; 4] = [115, 84, 69, 82];
const tEXt: [u8; 4] = [116, 69, 88, 116];
const tIME: [u8; 4] = [116, 73, 77, 69];
const tRNS: [u8; 4] = [116, 82, 78, 83];
const zTXt: [u8; 4] = [122, 84, 88, 116];

#[derive(Error, Debug)]
pub enum PNGError {
    #[error("The given file is not a png file")]
    NotAPng,
    #[error("Expected a IHDR block, found {:?}", .0)]
    ExpectedIHDR(Chunk),
    #[error("Unknown Chunktype: {:?}", String::from_utf8_lossy(.0))]
    UnknownChunk([u8; 4]),
    #[error("Mismatched CRC32, expected 0x{expected:0>8x}, found 0x{found:0>8x}")]
    MismatchedChecksum { expected: u32, found: u32 },
    #[error("Unexpected block length, expected 0x{expected:0>8x}, found 0x{found:0>8x}")]
    IncorrectChunkLengthExpectedExactly { expected: usize, found: usize },
    #[error("IEND chunk must not contain data")]
    NonEmptyIEND,
}

pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<()> {
    let mut file_contents = vec![];
    fs::File::open(&path)
        .with_context(|| format!("reading png data from {}", path.as_ref().display()))?
        .read_to_end(&mut file_contents)?;
    decode(&file_contents)
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum Chunk {
    /// Image Header
    IHDR(chunks::ImageHeader),
    /// Color Palette
    PLTE(chunks::Palette),
    /// Image Data
    IDAT,
    /// Image End
    IEND,
    cHRM,
    /// Digital Signatures
    dSIG,
    /// Exif Metadata
    eXIf,
    gAMA,
    /// Color Histogram
    hIST,
    /// ICC color profile
    iCCP,
    iTXt,
    pHYs,
    sBIT,
    sPLT,
    sRGB,
    sTER,
    tEXt,
    tIME,
    tRNS,
    zTXt,
}

pub fn decode(bytes: &[u8]) -> Result<()> {
    let mut reader = Cursor::new(bytes);

    let mut signature = [0; 8];
    reader.read_exact(&mut signature)?;

    if signature != PNG_HEADER {
        return Err(PNGError::NotAPng.into());
    }

    let ihdr_chunk = read_chunk(&mut reader)?;

    if let Chunk::IHDR(image_header) = ihdr_chunk {
        println!("{image_header:?}");
    } else {
        return Err(PNGError::ExpectedIHDR(ihdr_chunk).into());
    }

    loop {
        let chunk = read_chunk(&mut reader)?;
        println!("chunk {:?}", chunk);

        match chunk {
            Chunk::IEND => break,
            _ => {},
        }
    }

    Ok(())
}

fn read_chunk<R: Read>(reader: &mut R) -> Result<Chunk> {
    let mut length_bytes = [0; 4];
    reader.read_exact(&mut length_bytes)?;
    let length = u32::from_be_bytes(length_bytes) as usize;

    let mut chunk_name_bytes = [0; 4];
    reader.read_exact(&mut chunk_name_bytes)?;

    let mut data = vec![0; length];
    reader.read_exact(data.as_mut_slice())?;

    let mut crc_bytes = [0; 4];
    reader.read_exact(&mut crc_bytes)?;
    let expected_crc = u32::from_be_bytes(crc_bytes);

    let computed_crc = IncrementalCRC32::new()
        .update(&chunk_name_bytes)
        .update(&data)
        .finish();

    if expected_crc != computed_crc {
        return Err(PNGError::MismatchedChecksum {
            expected: expected_crc,
            found: computed_crc,
        }
        .into());
    }

    let chunk = match chunk_name_bytes {
        IHDR => {
            if length != 13 {
                return Err(PNGError::IncorrectChunkLengthExpectedExactly {
                    expected: 13,
                    found: length,
                }
                .into());
            }

            Chunk::IHDR(chunks::ImageHeader::new(
                u32::from_be_bytes(data[0..4].try_into().unwrap()),
                u32::from_be_bytes(data[4..8].try_into().unwrap()),
                data[8],
                data[9].try_into()?,
                data[10],
                data[11],
                data[12].try_into()?,
            )?)
        },
        PLTE => Chunk::PLTE(chunks::Palette::new(&data)?),
        IDAT => Chunk::IDAT,
        IEND => {
            if length != 0 {
                return Err(PNGError::NonEmptyIEND.into());
            }

            Chunk::IEND
        },
        cHRM => Chunk::cHRM,
        dSIG => Chunk::dSIG,
        eXIF => Chunk::eXIf,
        gAMA => Chunk::gAMA,
        hIST => Chunk::hIST,
        iCCP => Chunk::iCCP,
        iTXt => Chunk::iTXt,
        pHYs => Chunk::pHYs,
        sBIT => Chunk::sBIT,
        sPLT => Chunk::sPLT,
        sRGB => Chunk::sRGB,
        sTER => Chunk::sTER,
        tEXt => Chunk::tEXt,
        tIME => Chunk::tIME,
        tRNS => Chunk::tRNS,
        zTXt => Chunk::zTXt,
        _ => return Err(PNGError::UnknownChunk(chunk_name_bytes).into()),
    };

    Ok(chunk)
}
