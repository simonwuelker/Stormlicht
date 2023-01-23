#[derive(Clone, Copy, Debug)]
pub enum PixelFormat {
    GrayScale,
    RGB8,
    RGBA8,
}

impl PixelFormat {
    /// Return the number of bytes taken up by each pixel in the given format
    pub fn pixel_size(&self) -> usize {
        match &self {
            Self::GrayScale => 1,
            Self::RGB8 => 3,
            Self::RGBA8 => 4,
        }
    }
}
