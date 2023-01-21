pub mod png;

#[derive(Clone, Copy, Debug)]
pub enum PixelFormat {
    GrayScale,
    RGB8,
    RGBA8,
}
pub struct Image {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub format: PixelFormat,
}

impl Image {
    pub fn new(data: Vec<u8>, width: u32, height: u32, format: PixelFormat) -> Self {
        Self {
            data,
            width,
            height,
            format,
        }
    }
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
