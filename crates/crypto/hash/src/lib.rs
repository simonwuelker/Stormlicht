#![feature(cfg_match)]

mod adler32;
mod crc32;
mod md5;
mod sha;

pub use adler32::{adler32, Adler32Hasher};
pub use md5::Md5;
pub use sha::{Sha224, Sha256};
pub use {crc32::crc32, crc32::Crc32Hasher};

/// Something that is able to calculate a checksum over arbitrary bytes.
///
/// Implementors of this trait do not provide any kind of security guarantees.
/// For a cryptographically secure equivalent, use the [CryptographicHashAlgorithm] trait instead.
pub trait HashAlgorithm: Default {
    const BLOCK_SIZE_IN: usize;
    const BLOCK_SIZE_OUT: usize;

    fn update(&mut self, data: &[u8]);
    fn finish(self) -> [u8; Self::BLOCK_SIZE_OUT];

    fn hash(data: &[u8]) -> [u8; Self::BLOCK_SIZE_OUT] {
        let mut hasher = Self::default();
        hasher.update(data);
        hasher.finish()
    }
}

/// An algorithm suitable as a cryptographic hash function
///
/// Specifically, this implies that that the function
/// cannot trivially be inverted.
///
/// This is a marker trait, no functionality beyond [HashAlgorithm] is required.
pub trait CryptographicHashAlgorithm: HashAlgorithm {}
