//! SHA ([RFC 4634](https://www.rfc-editor.org/rfc/rfc4634)) hash implementation
//!
//! Note that this is not entirely spec-compliant. We do not support hashing
//! data with a length that isn't a multiple of 8 bits.

use crate::{CryptographicHashAlgorithm, HashAlgorithm};

const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

const PADDING: [u8; 64] = [
    0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

const SHA224_INITIAL: [u32; 8] = [
    0xc1059ed8, 0x367cd507, 0x3070dd17, 0xf70e5939, 0xffc00b31, 0x68581511, 0x64f98fa7, 0xbefa4fa4,
];

const SHA256_INITIAL: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];

#[inline]
#[must_use]
fn ch(x: u32, y: u32, z: u32) -> u32 {
    (x & y) | (!x & z)
}

#[inline]
#[must_use]
fn maj(x: u32, y: u32, z: u32) -> u32 {
    (x & y) ^ (x & z) ^ (y & z)
}

#[inline]
#[must_use]
fn bsig0(x: u32) -> u32 {
    x.rotate_right(2) ^ x.rotate_right(13) ^ x.rotate_right(22)
}

#[inline]
#[must_use]
fn bsig1(x: u32) -> u32 {
    x.rotate_right(6) ^ x.rotate_right(11) ^ x.rotate_right(25)
}

#[inline]
#[must_use]
fn ssig0(x: u32) -> u32 {
    x.rotate_right(7) ^ x.rotate_right(18) ^ (x >> 3)
}

#[inline]
#[must_use]
fn ssig1(x: u32) -> u32 {
    x.rotate_right(17) ^ x.rotate_right(19) ^ (x >> 10)
}

// NOTE: Internally, Sha224 is pretty much the same as SHA-256 so we just wrap
// a SHA-256 hasher.
#[derive(Clone, Copy, Debug)]
/// SHA-224 Hasher, as defined in [RFC 3874](https://www.rfc-editor.org/rfc/rfc3874)
pub struct Sha224(Sha256);

impl Default for Sha224 {
    fn default() -> Self {
        Self(Sha256 {
            state: SHA224_INITIAL,
            buffer: [0; 64],
            buffer_ptr: 0,
            num_bytes_consumed: 0,
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Sha256 {
    state: [u32; 8],
    buffer: [u8; 64],
    buffer_ptr: usize,
    num_bytes_consumed: u64,
}

impl Default for Sha256 {
    fn default() -> Self {
        Self {
            state: SHA256_INITIAL,
            buffer: [0; 64],
            buffer_ptr: 0,
            num_bytes_consumed: 0,
        }
    }
}

impl HashAlgorithm for Sha224 {
    const BLOCK_SIZE_IN: usize = 64;
    const BLOCK_SIZE_OUT: usize = 28;

    fn update(&mut self, bytes: &[u8]) {
        self.0.update(bytes);
    }

    fn finish(self) -> [u8; 28] {
        let sha256_hash = self.0.finish();
        sha256_hash[..28].try_into().expect("slice has length 28")
    }
}

impl CryptographicHashAlgorithm for Sha224 {}

impl HashAlgorithm for Sha256 {
    const BLOCK_SIZE_IN: usize = 64;
    const BLOCK_SIZE_OUT: usize = 32;

    fn update(&mut self, data: &[u8]) {
        let bytes_to_fill = 64 - self.buffer_ptr;
        if data.len() < bytes_to_fill {
            // Not enough to get a full chunk of 64 bytes, just update the buffer and call it a day
            self.buffer[self.buffer_ptr..self.buffer_ptr + data.len()].copy_from_slice(data);
            self.buffer_ptr += data.len();
            return;
        }

        // At this point, we have at least enough bytes to fill the buffer once
        self.buffer[self.buffer_ptr..].copy_from_slice(&data[..bytes_to_fill]);
        self.step();

        let chunks = data[bytes_to_fill..].chunks_exact(64);
        let remaining_bytes = chunks.remainder();
        for chunk in chunks {
            // TODO we shouldn't need to copy here (see md5 as well)
            self.buffer.copy_from_slice(chunk);
            self.step();
        }

        // Copy the remaining bytes into the buffer, the next `update()`
        // call will take care of them
        self.buffer[..remaining_bytes.len()].copy_from_slice(remaining_bytes);
        self.buffer_ptr = remaining_bytes.len();
    }

    fn finish(mut self) -> [u8; Self::BLOCK_SIZE_OUT] {
        // Important to get the length (in bits) now *before* we consume any padding
        let length: u64 = (self.num_bytes_consumed + self.buffer_ptr as u64) * 8;

        let needed_bytes = 64 - self.buffer_ptr;
        self.buffer[self.buffer_ptr..].copy_from_slice(&PADDING[..needed_bytes]);

        // We want to pad to be 8 bytes short of a full buffer. If we are currently
        // *less* than (or equal to) 8 bytes short, we need to fill the buffer, to a step and
        // fill the buffer *again*
        if needed_bytes <= 8 {
            // Need to fill a whole new block
            self.step();
            self.buffer[..56].fill(0);
        }

        // The last 8 bytes in the buffer are the number of bytes consumed
        // (in weird endianness)
        // let length_bits = length.to_le_bytes();
        self.buffer[56..64].copy_from_slice(&length.to_be_bytes());

        // At this point the buffer is completely filled, perform one final step
        self.step();

        let mut hash = [0; 32];
        hash[0..4].copy_from_slice(&self.state[0].to_be_bytes());
        hash[4..8].copy_from_slice(&self.state[1].to_be_bytes());
        hash[8..12].copy_from_slice(&self.state[2].to_be_bytes());
        hash[12..16].copy_from_slice(&self.state[3].to_be_bytes());
        hash[16..20].copy_from_slice(&self.state[4].to_be_bytes());
        hash[20..24].copy_from_slice(&self.state[5].to_be_bytes());
        hash[24..28].copy_from_slice(&self.state[6].to_be_bytes());
        hash[28..32].copy_from_slice(&self.state[7].to_be_bytes());
        hash
    }
}

impl CryptographicHashAlgorithm for Sha256 {}

impl Sha256 {
    fn step(&mut self) {
        let mut w = [0; 64];
        for (index, word_bytes) in self.buffer.chunks_exact(4).enumerate() {
            w[index] = u32::from_be_bytes(word_bytes.try_into().unwrap());
        }

        for t in 16..64 {
            w[t] = ssig1(w[t - 2])
                .wrapping_add(w[t - 7])
                .wrapping_add(ssig0(w[t - 15]))
                .wrapping_add(w[t - 16]);
        }

        let mut a = self.state[0];
        let mut b = self.state[1];
        let mut c = self.state[2];
        let mut d = self.state[3];
        let mut e = self.state[4];
        let mut f = self.state[5];
        let mut g = self.state[6];
        let mut h = self.state[7];

        for t in 0..64 {
            let t1 = h
                .wrapping_add(bsig1(e))
                .wrapping_add(ch(e, f, g))
                .wrapping_add(K[t])
                .wrapping_add(w[t]);
            let t2 = bsig0(a).wrapping_add(maj(a, b, c));

            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }

        self.state[0] = self.state[0].wrapping_add(a);
        self.state[1] = self.state[1].wrapping_add(b);
        self.state[2] = self.state[2].wrapping_add(c);
        self.state[3] = self.state[3].wrapping_add(d);
        self.state[4] = self.state[4].wrapping_add(e);
        self.state[5] = self.state[5].wrapping_add(f);
        self.state[6] = self.state[6].wrapping_add(g);
        self.state[7] = self.state[7].wrapping_add(h);

        self.num_bytes_consumed += 4;
        self.buffer_ptr = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha224() {
        assert_eq!(
            Sha224::hash(b"abc"),
            [
                0x23, 0x09, 0x7d, 0x22, 0x34, 0x05, 0xd8, 0x22, 0x86, 0x42, 0xa4, 0x77, 0xbd, 0xa2,
                0x55, 0xb3, 0x2a, 0xad, 0xbc, 0xe4, 0xbd, 0xa0, 0xb3, 0xf7, 0xe3, 0x6c, 0x9d, 0xa7
            ]
        );

        assert_eq!(
            Sha224::hash(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
            [
                0x75, 0x38, 0x8b, 0x16, 0x51, 0x27, 0x76, 0xcc, 0x5d, 0xba, 0x5d, 0xa1, 0xfd, 0x89,
                0x01, 0x50, 0xb0, 0xc6, 0x45, 0x5c, 0xb4, 0xf5, 0x8b, 0x19, 0x52, 0x52, 0x25, 0x25
            ]
        );
    }

    #[test]
    fn test_sha256() {
        assert_eq!(
            Sha256::hash(b"abc"),
            [
                0xBA, 0x78, 0x16, 0xBF, 0x8F, 0x01, 0xCF, 0xEA, 0x41, 0x41, 0x40, 0xDE, 0x5D, 0xAE,
                0x22, 0x23, 0xB0, 0x03, 0x61, 0xA3, 0x96, 0x17, 0x7A, 0x9C, 0xB4, 0x10, 0xFF, 0x61,
                0xF2, 0x00, 0x15, 0xAD
            ]
        );

        assert_eq!(
            Sha256::hash(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
            [
                0x24, 0x8D, 0x6A, 0x61, 0xD2, 0x06, 0x38, 0xB8, 0xE5, 0xC0, 0x26, 0x93, 0x0C, 0x3E,
                0x60, 0x39, 0xA3, 0x3C, 0xE4, 0x59, 0x64, 0xFF, 0x21, 0x67, 0xF6, 0xEC, 0xED, 0xD4,
                0x19, 0xDB, 0x06, 0xC1
            ]
        );
    }
}
