//! [PLTE](https://www.w3.org/TR/png/#11PLTE) chunk

#[derive(Clone, Copy, Debug)]
pub enum PaletteError {
    InvalidPaletteSize,
}

#[derive(Clone, Copy, Debug)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

#[derive(Clone, Debug)]
pub struct Palette {
    pub colors: Vec<Color>,
}

impl Palette {
    pub fn new(bytes: &[u8]) -> Result<Self, PaletteError> {
        if bytes.len() % 3 != 0 {
            return Err(PaletteError::InvalidPaletteSize);
        }

        let n_colors = bytes.len() / 3;

        let mut colors = Vec::with_capacity(n_colors);
        for i in 0..n_colors {
            colors.push(Color {
                red: bytes[i * 3],
                green: bytes[i * 3 + 1],
                blue: bytes[i * 3 + 2],
            })
        }

        Ok(Self { colors })
    }
}
