//! <https://encoding.spec.whatwg.org>

#![feature(ascii_char)]

mod decoder;
mod euc_jp;
mod euc_kr;

mod encodings {
    include!(concat!(env!("OUT_DIR"), "/encodings.rs"));
}

#[allow(dead_code)] // Not all tables are used yet
mod index {
    include!(concat!(env!("OUT_DIR"), "/indexes.rs"));
}

pub use encodings::Encoding;

pub use decoder::{Context, DecodeError, DecodeResult, Decoder};

///<https://encoding.spec.whatwg.org/#bom-sniff>
#[must_use]
pub fn bom_sniff(bytes: &[u8]) -> Option<Encoding> {
    // 1. Let BOM be the result of peeking 3 bytes from ioQueue, converted to a byte sequence.

    // 2. For each of the rows in the table below, starting with the first one and going down,
    // if BOM starts with the bytes given in the first column, then return the encoding given
    // in the cell in the second column of that row. Otherwise, return null.
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        Some(Encoding::UTF_8)
    } else if bytes.starts_with(&[0xFE, 0xFF]) {
        Some(Encoding::UTF_16BE)
    } else if bytes.starts_with(&[0xFF, 0xFE]) {
        Some(Encoding::UTF_16LE)
    } else {
        None
    }
}

/// <https://encoding.spec.whatwg.org/#decode>
pub fn decode(mut bytes: &[u8], mut encoding: Encoding) {
    // 1. Let BOMEncoding be the result of BOM sniffing ioQueue.
    let bom_encoding = bom_sniff(bytes);

    // 2. If BOMEncoding is non-null:
    if let Some(bom_encoding) = bom_encoding {
        // 1. Set encoding to BOMEncoding.
        encoding = bom_encoding;

        // 2. Read three bytes from ioQueue, if BOMEncoding is UTF-8;
        // otherwise read two bytes. (Do nothing with those bytes.)
        if bom_encoding == Encoding::UTF_8 {
            bytes = &bytes[3..];
        } else {
            bytes = &bytes[2..];
        }
    }

    todo!()
}
