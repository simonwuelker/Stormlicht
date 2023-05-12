//! Used to describe the memory layout that should be painted into

use crate::Color;

type Pixel = u32;

/// The target surface that content should be drawn to
#[derive(Clone, Debug)]
pub struct Buffer {
    width: usize,
    height: usize,
    data: Vec<Pixel>,
}

impl Buffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            data: vec![0; width * height],
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, pixel: Pixel) {
        debug_assert!(x < self.width);
        debug_assert!(y < self.height);

        let index = y * self.width + x;
        self.data[index] = pixel;
    }

    pub fn data(&self) -> &[Pixel] {
        &self.data
    }

    pub fn resize(&mut self, new_width: usize, new_height: usize) {
        self.width = new_width;
        self.height = new_height;
        self.data.resize(new_width * new_height, 0);
    }

    pub fn clear(&mut self, clear_color: Color) {
        self.data.fill(clear_color.into());
    }
}
