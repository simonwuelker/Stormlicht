pub type Color = (u8, u8, u8);

pub struct PixelBuffer<'a> {
    data: &'a mut [u8],
    width: usize,
    height: usize,
    stride: usize,
}

/// Implementors may override default implementations if more performant solutions are available.
pub trait RendererTarget {
    fn set_pixel(&mut self, x: usize, y: usize, color: Color);
    fn width(&self) -> usize;
    fn height(&self) -> usize;

    /// Fill the entire target with a specific color
    fn fill(&mut self, color: Color) {
        for x in 0..self.width() {
            for y in 0..self.height() {
                self.set_pixel(x, y, color);
            }
        }
    }
}

impl<'a> PixelBuffer<'a> {
    pub fn new(data: &'a mut [u8], width: usize, height: usize, stride: usize) -> Self {
        Self {
            data: data,
            width: width,
            height: height,
            stride: stride,
        }
    }
}

impl<'a> RendererTarget for PixelBuffer<'a> {
    fn set_pixel(&mut self, x: usize, y: usize, color: Color) {
        let index = y * self.stride + x * 4;
        self.data[index + 0] = color.2;
        self.data[index + 1] = color.1;
        self.data[index + 2] = color.0;
        self.data[index + 3] = 0; // unused
    }

    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    fn fill(&mut self, color: Color) {
        for i in 0..(self.stride * self.height) {
            self.data[4 * i] = color.2;
            self.data[4 * i + 1] = color.1;
            self.data[4 * i + 2] = color.0;
        }
    }
}
