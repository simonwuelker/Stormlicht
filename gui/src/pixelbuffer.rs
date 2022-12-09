pub type Color = (u8, u8, u8);
pub type Point = (usize, usize);

pub struct PixelBuffer<'a> {
    data: &'a mut [u8],
    width: usize,
    height: usize,
    stride: usize,
}

pub struct RendererTargetView<T: RendererTarget> {
    offset: Point,
    inner: T,
    width: usize,
    height: usize,
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

    fn line(&mut self, from: Point, to: Point, color: Color) {
        assert!(0 < from.0 && from.0 < self.width(), "X Coordinate outside of canvas");
        assert!(0 < from.1 && from.1 < self.height(), "Y Coordinate outside of canvas");
        assert!(from.0 <= to.0 && from.1 < to.1, "'to' must be further from origin than 'from'");

        let d_x = to.0 - from.0;
        let d_y = to.1 - from.1;

        for x in 0..d_x {
            let y = (d_y * x) / d_x;
            self.set_pixel(from.0 + x, from.1 + y, color);
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

impl<T: RendererTarget> RendererTarget for RendererTargetView<T> {
    fn set_pixel(&mut self, x: usize, y: usize, color: Color) {
        if x > self.width() || y > self.height() {
            return;
        }
        self.inner.set_pixel(self.offset.0 + x, self.offset.1 + y, color);
    }

    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }
}

impl<T: RendererTarget> RendererTargetView<T> {
    pub fn new(inner: T, offset: Point, width: usize, height: usize) -> Self {
        Self {
            offset: offset,
            inner: inner,
            width: width,
            height: height,
        }
    }

    pub fn release(self) -> T {
        self.inner
    }
}
