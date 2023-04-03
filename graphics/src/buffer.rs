use crate::consts;

#[derive(Clone, Debug)]
pub struct Buffer<'a, M: BufferLayout> {
    /// The raw bytes contained by the buffer
    data: &'a [u8],

    /// Describes how to interpret [Self::data]
    layout: M,
}

pub trait BufferLayout {
    /// Buffer width in pixels
    fn width(&self) -> usize;

    /// Buffer height in pixels
    fn height(&self) -> usize;

    /// Buffer width in tiles
    fn tile_width(&self) -> usize {
        (self.width() + consts::TILE_SIZE - 1) / consts::TILE_SIZE
    }

    /// Buffer height in teils
    fn tile_height(&self) -> usize {
        (self.height() + consts::TILE_SIZE - 1) / consts::TILE_SIZE
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LinearLayout {
    width: usize,
    height: usize,
}

impl BufferLayout for LinearLayout {
    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }
}
