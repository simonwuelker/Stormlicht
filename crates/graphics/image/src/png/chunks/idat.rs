//! [IDAT](https://www.w3.org/TR/png/#11IDAT) chunk

use std::fmt;

/// Wrapper type around `Vec<u8>` so we can easily generate a debugimpl for [Chunk](crate::png::Chunk)
#[derive(Clone)]
pub struct ImageData(Vec<u8>);

impl ImageData {
    #[must_use]
    pub fn new(data: Vec<u8>) -> Self {
        Self(data)
    }

    #[must_use]
    pub fn bytes(self) -> Vec<u8> {
        self.0
    }
}
impl fmt::Debug for ImageData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} bytes", self.0.len())
    }
}
