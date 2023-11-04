//! Md5 [RFC 1321](https://datatracker.ietf.org/doc/html/rfc1321) implementation

const T: [u32; 64] = [
    0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee, 0xf57c0faf, 0x4787c62a, 0xa8304613, 0xfd469501,
    0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be, 0x6b901122, 0xfd987193, 0xa679438e, 0x49b40821,
    0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa, 0xd62f105d, 0x02441453, 0xd8a1e681, 0xe7d3fbc8,
    0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed, 0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a,
    0xfffa3942, 0x8771f681, 0x6d9d6122, 0xfde5380c, 0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70,
    0x289b7ec6, 0xeaa127fa, 0xd4ef3085, 0x04881d05, 0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665,
    0xf4292244, 0x432aff97, 0xab9423a7, 0xfc93a039, 0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1,
    0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1, 0xf7537e82, 0xbd3af235, 0x2ad7d2bb, 0xeb86d391,
];

const PADDING: [u8; 64] = [
    0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

macro_rules! F {
    ($x: expr, $y: expr, $z: expr) => {
        ($x & $y) | (!$x & $z)
    };
}

macro_rules! G {
    ($x: expr, $y: expr, $z: expr) => {
        ($x & $z) | ($y & !$z)
    };
}

macro_rules! H {
    ($x: expr, $y: expr, $z: expr) => {
        $x ^ $y ^ $z
    };
}

macro_rules! I {
    ($x: expr, $y: expr, $z: expr) => {
        $y ^ ($x | !$z)
    };
}

macro_rules! step {
    ( $fn:ident, $a: expr, $b: expr, $c: expr, $d: expr, $value: expr, $i: expr, $rotate_by: expr) => {{
        $a = $b.wrapping_add(
            ($a.wrapping_add($fn!($b, $c, $d))
                .wrapping_add($value as u32)
                .wrapping_add(T[$i - 1]))
            .rotate_left($rotate_by),
        );
    }};
}

pub struct Md5Hasher {
    a: u32,
    b: u32,
    c: u32,
    d: u32,
    buffer: [u8; 64],
    buffer_ptr: usize,
    num_bytes_consumed: u64,
}

impl Default for Md5Hasher {
    fn default() -> Self {
        Self {
            a: u32::from_le_bytes([0x01, 0x23, 0x45, 0x67]),
            b: u32::from_le_bytes([0x89, 0xab, 0xcd, 0xef]),
            c: u32::from_le_bytes([0xfe, 0xdc, 0xba, 0x98]),
            d: u32::from_le_bytes([0x76, 0x54, 0x32, 0x10]),
            buffer: [0; 64],
            buffer_ptr: 0,
            num_bytes_consumed: 0,
        }
    }
}

impl Md5Hasher {
    pub fn update(&mut self, bytes: &[u8]) {
        let bytes_to_fill = 64 - self.buffer_ptr;
        if bytes.len() < bytes_to_fill {
            // Not enough to get a full chunk of 64 bytes, just update the buffer and call it a day
            self.buffer[self.buffer_ptr..self.buffer_ptr + bytes.len()].copy_from_slice(bytes);
            self.buffer_ptr += bytes.len();
            return;
        }

        // At this point, we have at least enough bytes to fill the buffer once
        self.buffer[self.buffer_ptr..].copy_from_slice(&bytes[..bytes_to_fill]);
        self.step();

        let chunks = bytes[bytes_to_fill..].chunks_exact(64);
        let remaining_bytes = chunks.remainder();
        for chunk in chunks {
            // TODO we shouldn't really need to copy here
            self.buffer.copy_from_slice(chunk);
            self.step();
        }

        // Copy the remaining bytes into the buffer, the next `update()`
        // call will take care of them
        self.buffer[..remaining_bytes.len()].copy_from_slice(remaining_bytes);
        self.buffer_ptr = remaining_bytes.len();
    }

    pub fn finish(&mut self) -> u128 {
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
        self.buffer[56..64].copy_from_slice(&length.to_le_bytes());

        // At this point the buffer is completely filled, perform one final step
        self.step();

        // Build the hash value from a, b, c, d
        let mut hash_bytes = [0; 16];
        hash_bytes[0x00..0x04].copy_from_slice(&self.a.to_le_bytes());
        hash_bytes[0x04..0x08].copy_from_slice(&self.b.to_le_bytes());
        hash_bytes[0x08..0x0C].copy_from_slice(&self.c.to_le_bytes());
        hash_bytes[0x0C..0x10].copy_from_slice(&self.d.to_le_bytes());

        u128::from_be_bytes(hash_bytes)
    }

    // Perform the 4-round md5 algorithm once.
    // This assumes that the buffer has been filled.
    fn step(&mut self) {
        let mut chunk = [0; 16];

        for (index, u32_bytes) in self.buffer.chunks_exact(4).enumerate() {
            chunk[index] = u32::from_le_bytes(u32_bytes.try_into().unwrap());
        }

        let old_a = self.a;
        let old_b = self.b;
        let old_c = self.c;
        let old_d = self.d;

        // Round 1
        step!(F, self.a, self.b, self.c, self.d, chunk[0x00], 0x01, 7);
        step!(F, self.d, self.a, self.b, self.c, chunk[0x01], 0x02, 12);
        step!(F, self.c, self.d, self.a, self.b, chunk[0x02], 0x03, 17);
        step!(F, self.b, self.c, self.d, self.a, chunk[0x03], 0x04, 22);

        step!(F, self.a, self.b, self.c, self.d, chunk[0x04], 0x05, 7);
        step!(F, self.d, self.a, self.b, self.c, chunk[0x05], 0x06, 12);
        step!(F, self.c, self.d, self.a, self.b, chunk[0x06], 0x07, 17);
        step!(F, self.b, self.c, self.d, self.a, chunk[0x07], 0x08, 22);

        step!(F, self.a, self.b, self.c, self.d, chunk[0x08], 0x09, 7);
        step!(F, self.d, self.a, self.b, self.c, chunk[0x09], 0x0A, 12);
        step!(F, self.c, self.d, self.a, self.b, chunk[0x0A], 0x0B, 17);
        step!(F, self.b, self.c, self.d, self.a, chunk[0x0B], 0x0C, 22);

        step!(F, self.a, self.b, self.c, self.d, chunk[0x0C], 0x0D, 7);
        step!(F, self.d, self.a, self.b, self.c, chunk[0x0D], 0x0E, 12);
        step!(F, self.c, self.d, self.a, self.b, chunk[0x0E], 0x0F, 17);
        step!(F, self.b, self.c, self.d, self.a, chunk[0x0F], 0x10, 22);

        // Round 2
        step!(G, self.a, self.b, self.c, self.d, chunk[0x01], 0x11, 5);
        step!(G, self.d, self.a, self.b, self.c, chunk[0x06], 0x12, 9);
        step!(G, self.c, self.d, self.a, self.b, chunk[0x0B], 0x13, 14);
        step!(G, self.b, self.c, self.d, self.a, chunk[0x00], 0x14, 20);

        step!(G, self.a, self.b, self.c, self.d, chunk[0x05], 0x15, 5);
        step!(G, self.d, self.a, self.b, self.c, chunk[0x0A], 0x16, 9);
        step!(G, self.c, self.d, self.a, self.b, chunk[0x0F], 0x17, 14);
        step!(G, self.b, self.c, self.d, self.a, chunk[0x04], 0x18, 20);

        step!(G, self.a, self.b, self.c, self.d, chunk[0x09], 0x19, 5);
        step!(G, self.d, self.a, self.b, self.c, chunk[0x0E], 0x1A, 9);
        step!(G, self.c, self.d, self.a, self.b, chunk[0x03], 0x1B, 14);
        step!(G, self.b, self.c, self.d, self.a, chunk[0x08], 0x1C, 20);

        step!(G, self.a, self.b, self.c, self.d, chunk[0x0D], 0x1D, 5);
        step!(G, self.d, self.a, self.b, self.c, chunk[0x02], 0x1E, 9);
        step!(G, self.c, self.d, self.a, self.b, chunk[0x07], 0x1F, 14);
        step!(G, self.b, self.c, self.d, self.a, chunk[0x0C], 0x20, 20);

        // Round 3
        step!(H, self.a, self.b, self.c, self.d, chunk[0x05], 0x21, 4);
        step!(H, self.d, self.a, self.b, self.c, chunk[0x08], 0x22, 11);
        step!(H, self.c, self.d, self.a, self.b, chunk[0x0B], 0x23, 16);
        step!(H, self.b, self.c, self.d, self.a, chunk[0x0E], 0x24, 23);

        step!(H, self.a, self.b, self.c, self.d, chunk[0x01], 0x25, 4);
        step!(H, self.d, self.a, self.b, self.c, chunk[0x04], 0x26, 11);
        step!(H, self.c, self.d, self.a, self.b, chunk[0x07], 0x27, 16);
        step!(H, self.b, self.c, self.d, self.a, chunk[0x0A], 0x28, 23);

        step!(H, self.a, self.b, self.c, self.d, chunk[0x0D], 0x29, 4);
        step!(H, self.d, self.a, self.b, self.c, chunk[0x00], 0x2A, 11);
        step!(H, self.c, self.d, self.a, self.b, chunk[0x03], 0x2B, 16);
        step!(H, self.b, self.c, self.d, self.a, chunk[0x06], 0x2C, 23);

        step!(H, self.a, self.b, self.c, self.d, chunk[0x09], 0x2D, 4);
        step!(H, self.d, self.a, self.b, self.c, chunk[0x0C], 0x2E, 11);
        step!(H, self.c, self.d, self.a, self.b, chunk[0x0F], 0x2F, 16);
        step!(H, self.b, self.c, self.d, self.a, chunk[0x02], 0x30, 23);

        // Round 4
        step!(I, self.a, self.b, self.c, self.d, chunk[0x00], 0x31, 6);
        step!(I, self.d, self.a, self.b, self.c, chunk[0x07], 0x32, 10);
        step!(I, self.c, self.d, self.a, self.b, chunk[0x0E], 0x33, 15);
        step!(I, self.b, self.c, self.d, self.a, chunk[0x05], 0x34, 21);

        step!(I, self.a, self.b, self.c, self.d, chunk[0x0C], 0x35, 6);
        step!(I, self.d, self.a, self.b, self.c, chunk[0x03], 0x36, 10);
        step!(I, self.c, self.d, self.a, self.b, chunk[0x0A], 0x37, 15);
        step!(I, self.b, self.c, self.d, self.a, chunk[0x01], 0x38, 21);

        step!(I, self.a, self.b, self.c, self.d, chunk[0x08], 0x39, 6);
        step!(I, self.d, self.a, self.b, self.c, chunk[0x0F], 0x3A, 10);
        step!(I, self.c, self.d, self.a, self.b, chunk[0x06], 0x3B, 15);
        step!(I, self.b, self.c, self.d, self.a, chunk[0x0D], 0x3C, 21);

        step!(I, self.a, self.b, self.c, self.d, chunk[0x04], 0x3D, 6);
        step!(I, self.d, self.a, self.b, self.c, chunk[0x0B], 0x3E, 10);
        step!(I, self.c, self.d, self.a, self.b, chunk[0x02], 0x3F, 15);
        step!(I, self.b, self.c, self.d, self.a, chunk[0x09], 0x40, 21);

        self.a = self.a.wrapping_add(old_a);
        self.b = self.b.wrapping_add(old_b);
        self.c = self.c.wrapping_add(old_c);
        self.d = self.d.wrapping_add(old_d);

        self.buffer_ptr = 0;
        self.num_bytes_consumed += 64;
    }
}

/// Calculate the MD5 Hash for the given bytes.
///
/// This is a convenience function around creating a [Md5Hasher] instance.
pub fn md5(bytes: &[u8]) -> u128 {
    let mut hasher = Md5Hasher::default();
    hasher.update(bytes);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::md5;

    #[test]
    fn md5_test_vectors() {
        assert_eq!(md5(b""), 0xd41d8cd98f00b204e9800998ecf8427e);
        assert_eq!(md5(b"a"), 0x0cc175b9c0f1b6a831c399e269772661);
        assert_eq!(md5(b"abc"), 0x900150983cd24fb0d6963f7d28e17f72);
        assert_eq!(md5(b"message digest"), 0xf96b697d7cb7938d525a2f31aaf161d0);
        assert_eq!(
            md5(b"abcdefghijklmnopqrstuvwxyz"),
            0xc3fcd3d76192e4007dfb496cca67e13b
        );
        assert_eq!(
            md5(b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789"),
            0xd174ab98d277d9f5a5611c2c9f419d9f
        );
        assert_eq!(
            md5(
                b"12345678901234567890123456789012345678901234567890123456789012345678901234567890"
            ),
            0x57edf4a22be3c955ac49da2e2107b67a
        );
    }
}
