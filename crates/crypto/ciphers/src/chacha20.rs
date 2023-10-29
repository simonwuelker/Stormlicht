//! ChaCha20 ([RFC 7539](https://datatracker.ietf.org/doc/html/rfc7539)) implementation

macro_rules! quarter_round {
    ($a: expr, $b: expr, $c: expr, $d: expr) => {
        $a = $a.wrapping_add($b);
        $d ^= $a;
        $d = $d.rotate_left(16);

        $c = $c.wrapping_add($d);
        $b ^= $c;
        $b = $b.rotate_left(12);

        $a = $a.wrapping_add($b);
        $d ^= $a;
        $d = $d.rotate_left(8);

        $c = $c.wrapping_add($d);
        $b ^= $c;
        $b = $b.rotate_left(7);
    };
}

pub struct ChaCha20 {
    state: [u32; 16],
}

impl ChaCha20 {
    pub fn from_state(state: [u32; 16]) -> Self {
        Self { state: state }
    }

    fn quarter_round(&mut self, a: usize, b: usize, c: usize, d: usize) {
        quarter_round!(self.state[a], self.state[b], self.state[c], self.state[d]);
    }

    pub fn block(key: [u8; 32], nonce: [u8; 12], block_count: u32) -> [u8; 64] {
        let initial_state = [
            0x61707865,
            0x3320646e,
            0x79622d32,
            0x6b206574,
            u32::from_le_bytes(key[0x00..0x04].try_into().unwrap()),
            u32::from_le_bytes(key[0x04..0x08].try_into().unwrap()),
            u32::from_le_bytes(key[0x08..0x0C].try_into().unwrap()),
            u32::from_le_bytes(key[0x0C..0x10].try_into().unwrap()),
            u32::from_le_bytes(key[0x10..0x14].try_into().unwrap()),
            u32::from_le_bytes(key[0x14..0x18].try_into().unwrap()),
            u32::from_le_bytes(key[0x18..0x1C].try_into().unwrap()),
            u32::from_le_bytes(key[0x1C..0x20].try_into().unwrap()),
            block_count,
            u32::from_le_bytes(nonce[0x00..0x04].try_into().unwrap()),
            u32::from_le_bytes(nonce[0x04..0x08].try_into().unwrap()),
            u32::from_le_bytes(nonce[0x08..0x0C].try_into().unwrap()),
        ];
        let mut chacha20 = Self {
            state: initial_state,
        };

        for _ in 0..10 {
            // Column round
            chacha20.quarter_round(0, 4, 8, 12);
            chacha20.quarter_round(1, 5, 9, 13);
            chacha20.quarter_round(2, 6, 10, 14);
            chacha20.quarter_round(3, 7, 11, 15);

            // Diagonal round
            chacha20.quarter_round(0, 5, 10, 15);
            chacha20.quarter_round(1, 6, 11, 12);
            chacha20.quarter_round(2, 7, 8, 13);
            chacha20.quarter_round(3, 4, 9, 14);
        }

        for i in 0..16 {
            // Add the input words to the output words
            chacha20.state[i] = chacha20.state[i].wrapping_add(initial_state[i]);
        }

        let mut result = [0; 64];
        for i in 0..16 {
            // Serialize
            result[4 * i..4 * (i + 1)].copy_from_slice(&chacha20.state[i].to_le_bytes())
        }
        result
    }

    pub fn encrypt(
        key: [u8; 32],
        initial_counter: u32,
        nonce: [u8; 12],
        plaintext: &[u8],
    ) -> Vec<u8> {
        let chunks = plaintext.chunks_exact(64);
        let remainder = chunks.remainder();
        let mut ciphertext = Vec::with_capacity(plaintext.len());

        for (i, plaintext_chunk) in chunks.enumerate() {
            let key_stream = Self::block(key, nonce, initial_counter.wrapping_add(i as u32));
            for i in 0..64 {
                ciphertext.push(key_stream[i] ^ plaintext_chunk[i]);
            }
        }

        if !remainder.is_empty() {
            let i = plaintext.len() / 64;
            let key_stream = Self::block(key, nonce, initial_counter.wrapping_add(i as u32));
            for i in 0..remainder.len() {
                ciphertext.push(key_stream[i] ^ remainder[i]);
            }
        }
        ciphertext
    }

    /// Decrypt a ChaCha20 encoded byte stream.
    ///
    /// This is a convenience function that calls [encrypt()](encrypt)
    pub fn decrypt(
        key: [u8; 32],
        initial_counter: u32,
        nonce: [u8; 12],
        plaintext: &[u8],
    ) -> Vec<u8> {
        Self::encrypt(key, initial_counter, nonce, plaintext)
    }
}

#[cfg(test)]
mod tests {
    use super::ChaCha20;

    #[test]
    fn test_quarter_round() {
        let mut a = 0x11111111_u32;
        let mut b = 0x01020304_u32;
        let mut c = 0x9b8d6f43_u32;
        let mut d = 0x01234567_u32;

        quarter_round!(a, b, c, d);

        assert_eq!(a, 0xea2a92f4);
        assert_eq!(b, 0xcb1cf8ce);
        assert_eq!(c, 0x4581472e);
        assert_eq!(d, 0x5881c4bb);
    }

    #[test]
    fn test_chacha20_quarter_round() {
        let mut chacha20 = ChaCha20::from_state([
            0x879531e0, 0xc5ecf37d, 0x516461b1, 0xc9a62f8a, 0x44c20ef3, 0x3390af7f, 0xd9fc690b,
            0x2a5f714c, 0x53372767, 0xb00a5631, 0x974c541a, 0x359e9963, 0x5c971061, 0x3d631689,
            0x2098d9d6, 0x91dbd320,
        ]);
        chacha20.quarter_round(2, 7, 8, 13);

        assert_eq!(
            chacha20.state,
            [
                0x879531e0, 0xc5ecf37d, 0xbdb886dc, 0xc9a62f8a, 0x44c20ef3, 0x3390af7f, 0xd9fc690b,
                0xcfacafd2, 0xe46bea80, 0xb00a5631, 0x974c541a, 0x359e9963, 0x5c971061, 0xccc07c79,
                0x2098d9d6, 0x91dbd320,
            ]
        )
    }

    #[test]
    fn test_chacha20_block() {
        let key = [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
            0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b,
            0x1c, 0x1d, 0x1e, 0x1f,
        ];
        let nonce = [
            0x00, 0x00, 0x00, 0x09, 0x00, 0x00, 0x00, 0x4a, 0x00, 0x00, 0x00, 0x00,
        ];
        let block_count = 1;
        let block = ChaCha20::block(key, nonce, block_count);

        let expected_block = [
            0x10, 0xf1, 0xe7, 0xe4, 0xd1, 0x3b, 0x59, 0x15, 0x50, 0x0f, 0xdd, 0x1f, 0xa3, 0x20,
            0x71, 0xc4, 0xc7, 0xd1, 0xf4, 0xc7, 0x33, 0xc0, 0x68, 0x03, 0x04, 0x22, 0xaa, 0x9a,
            0xc3, 0xd4, 0x6c, 0x4e, 0xd2, 0x82, 0x64, 0x46, 0x07, 0x9f, 0xaa, 0x09, 0x14, 0xc2,
            0xd7, 0x05, 0xd9, 0x8b, 0x02, 0xa2, 0xb5, 0x12, 0x9c, 0xd1, 0xde, 0x16, 0x4e, 0xb9,
            0xcb, 0xd0, 0x83, 0xe8, 0xa2, 0x50, 0x3c, 0x4e,
        ];
        assert_eq!(block, expected_block);
    }

    #[test]
    fn test_chacha20_encrypt_decrypt() {
        let key = [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
            0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b,
            0x1c, 0x1d, 0x1e, 0x1f,
        ];
        let nonce = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x4a, 0x00, 0x00, 0x00, 0x00,
        ];
        let initial_counter = 1;
        // Baz Luhrmann - Everybody's Free
        let plaintext = b"Ladies and Gentlemen of the class of '99: If I could offer you only one tip for the future, sunscreen would be it.";

        let expected_ciphertext = [
            0x6e, 0x2e, 0x35, 0x9a, 0x25, 0x68, 0xf9, 0x80, 0x41, 0xba, 0x07, 0x28, 0xdd, 0x0d,
            0x69, 0x81, 0xe9, 0x7e, 0x7a, 0xec, 0x1d, 0x43, 0x60, 0xc2, 0x0a, 0x27, 0xaf, 0xcc,
            0xfd, 0x9f, 0xae, 0x0b, 0xf9, 0x1b, 0x65, 0xc5, 0x52, 0x47, 0x33, 0xab, 0x8f, 0x59,
            0x3d, 0xab, 0xcd, 0x62, 0xb3, 0x57, 0x16, 0x39, 0xd6, 0x24, 0xe6, 0x51, 0x52, 0xab,
            0x8f, 0x53, 0x0c, 0x35, 0x9f, 0x08, 0x61, 0xd8, 0x07, 0xca, 0x0d, 0xbf, 0x50, 0x0d,
            0x6a, 0x61, 0x56, 0xa3, 0x8e, 0x08, 0x8a, 0x22, 0xb6, 0x5e, 0x52, 0xbc, 0x51, 0x4d,
            0x16, 0xcc, 0xf8, 0x06, 0x81, 0x8c, 0xe9, 0x1a, 0xb7, 0x79, 0x37, 0x36, 0x5a, 0xf9,
            0x0b, 0xbf, 0x74, 0xa3, 0x5b, 0xe6, 0xb4, 0x0b, 0x8e, 0xed, 0xf2, 0x78, 0x5e, 0x42,
            0x87, 0x4d,
        ];

        assert_eq!(
            ChaCha20::encrypt(key, initial_counter, nonce, plaintext),
            expected_ciphertext
        );

        assert_eq!(
            ChaCha20::decrypt(key, initial_counter, nonce, &expected_ciphertext,),
            plaintext
        );
    }
}
