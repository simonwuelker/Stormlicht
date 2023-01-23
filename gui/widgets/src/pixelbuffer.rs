use font_rasterizer::target::{Point as RasterizerPoint, RasterizerTarget};

pub type Color = (u8, u8, u8);
pub type Point = (i16, i16);

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

    fn line(&mut self, from: Point, to: Point, color: Color) {
        let d_x = (to.0 - from.0).abs();
        let d_y = -(to.1 - from.1).abs();

        let s_x = if from.0 < to.0 { 1 } else { -1 };
        let s_y = if from.1 < to.1 { 1 } else { -1 };

        let mut error = d_x + d_y;

        let mut current = from;
        loop {
            debug_assert!(!current.0.is_negative() && !current.1.is_negative());
            self.set_pixel(current.0 as usize, current.1 as usize, color);

            if current == to {
                break;
            }

            let e2 = 2 * error;
            if e2 >= d_y {
                if current.0 == to.0 {
                    break;
                }
                error += d_y;
                current.0 += s_x;
            }

            if e2 <= d_x {
                if current.1 == to.1 {
                    break;
                }
                error += d_x;
                current.1 += s_y;
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

impl<'a> RasterizerTarget for PixelBuffer<'a> {
    fn width(&self) -> usize {
        RendererTarget::width(self)
    }
    fn height(&self) -> usize {
        RendererTarget::height(self)
    }
    fn line(&mut self, from: RasterizerPoint, to: RasterizerPoint) {
        RendererTarget::line(self, (from.x, from.y), (to.x, to.y), (0, 0, 0));
    }
}
