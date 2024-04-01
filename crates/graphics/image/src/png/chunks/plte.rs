//! [PLTE](https://www.w3.org/TR/png/#11PLTE) chunk

use std::ops::Index;

const PALETTE_MAX_SIZE: usize = 256 * 3;

#[derive(Clone, Copy, Debug)]
pub enum PaletteError {
    InvalidPaletteSize,
    TooLong,
}

#[derive(Clone, Debug)]
pub struct Palette {
    /// We always store 256 colors (the maximum), even if the actual
    /// size of the palette is less, to not have to worry about out of bounds errors
    colors: [u8; PALETTE_MAX_SIZE],
}

impl Palette {
    pub fn new(bytes: &[u8]) -> Result<Self, PaletteError> {
        if bytes.len() % 3 != 0 {
            return Err(PaletteError::InvalidPaletteSize);
        }

        if bytes.len() > PALETTE_MAX_SIZE {
            return Err(PaletteError::TooLong);
        }

        let mut colors = [0; PALETTE_MAX_SIZE];
        colors[..bytes.len()].copy_from_slice(bytes);

        let palette = Self { colors };
        Ok(palette)
    }
}

impl Index<u8> for Palette {
    type Output = [u8];

    fn index(&self, index: u8) -> &Self::Output {
        self.colors
            .chunks_exact(3)
            .nth(index as usize)
            .expect("Palette index out of bounds")
    }
}
