//! [PLTE](https://www.w3.org/TR/png/#11PLTE) chunk

use std::ops::Index;

use crate::texture::Rgbaf32;

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
    colors: [Rgbaf32; PALETTE_MAX_SIZE],
}

impl Palette {
    pub fn new(bytes: &[u8]) -> Result<Self, PaletteError> {
        let color_values = bytes.array_chunks::<3>();

        if !color_values.remainder().is_empty() {
            return Err(PaletteError::InvalidPaletteSize);
        }

        if color_values.len() > PALETTE_MAX_SIZE {
            return Err(PaletteError::TooLong);
        }

        let mut colors = [Rgbaf32::default(); PALETTE_MAX_SIZE];

        for (slot, [r, g, b]) in colors.iter_mut().zip(color_values) {
            *slot = Rgbaf32::rgb(*r as f32 / 255., *g as f32 / 255., *b as f32 / 255.);
        }

        let palette = Self { colors };
        Ok(palette)
    }
}

impl Index<u8> for Palette {
    type Output = Rgbaf32;

    fn index(&self, index: u8) -> &Self::Output {
        &self.colors[index as usize]
    }
}
