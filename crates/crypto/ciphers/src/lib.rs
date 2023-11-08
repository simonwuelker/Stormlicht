mod aes;
mod chacha20;

pub use aes::Aes128Cipher;
pub use chacha20::ChaCha20;

pub trait BlockCipher {
    type Block;
    type Key;

    fn new(key: Self::Key) -> Self
    where
        Self: Sized;
    fn encrypt_block(&mut self, input: Self::Block) -> Self::Block;
    fn decrypt_block(&mut self, output: Self::Block) -> Self::Block;
}
