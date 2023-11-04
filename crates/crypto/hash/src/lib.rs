#![feature(cfg_match)]

mod adler32;
mod crc32;
mod md5;
mod sha;

pub use adler32::{adler32, Adler32Hasher};
pub use md5::{md5, Md5Hasher};
pub use sha::{sha224, sha256, Sha224Hasher, Sha256Hasher};
pub use {crc32::crc32, crc32::Crc32Hasher};
