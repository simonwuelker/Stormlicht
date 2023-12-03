use ciphers::{Aes128Cipher, BlockCipher};

/// Similar to AES-CTR mode.
/// I think this is more or less equivalent to AES_CTR_DRBG but
/// it's more of a "good enough for now" thing.
/// It's `AES128(counter)`, seeded from `/dev/urandom`.
pub struct CryptographicRand {
    state: u128,
    key: Aes128Cipher,
}

impl CryptographicRand {
    /// Seed a new CPRNG from `/dev/urandom`.
    #[cfg(target_os = "linux")]
    pub fn new() -> Result<Self, std::io::Error> {
        use std::{fs, io::Read};

        let mut entropy_source = fs::File::open("/dev/urandom")?;

        let mut key = [0; 16];
        entropy_source.read_exact(&mut key)?;

        let mut inital_state = [0; 16];
        entropy_source.read_exact(&mut inital_state)?;

        Ok(Self {
            state: u128::from_ne_bytes(inital_state),
            key: Aes128Cipher::new(key),
        })
    }

    #[cfg(not(target_os = "linux"))]
    pub fn new() -> Result<Self, std::io::Error> {
        log::warn!("FIXME: Cannot initialize RNG securely on non-linx platform");

        Ok(Self {
            state: 0,
            key: Aes128Cipher::new([0; 16]),
        })
    }

    pub fn next_u128(&mut self) -> u128 {
        let rand = self.key.encrypt_block(self.state.to_ne_bytes());
        self.state = self.state.wrapping_add(1);
        u128::from_ne_bytes(rand)
    }

    pub fn next_u64(&mut self) -> u64 {
        (self.next_u128() & u64::MAX as u128) as u64
    }

    pub fn next_u32(&mut self) -> u32 {
        (self.next_u128() & u32::MAX as u128) as u32
    }

    pub fn next_u16(&mut self) -> u16 {
        (self.next_u128() & u16::MAX as u128) as u16
    }

    pub fn next_u8(&mut self) -> u8 {
        (self.next_u128() & u8::MAX as u128) as u8
    }
}
