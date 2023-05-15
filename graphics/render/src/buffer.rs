//! Used to describe the memory layout that should be painted into

use crate::{layer::Source, Color, Mask};
use math::Vec2D;

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

    /// Set the pixel at the given coordinates to the specified value.
    ///
    /// # Panics
    /// This function panics if the coordinates are outside of the bitmap
    pub fn set_pixel(&mut self, x: usize, y: usize, pixel: Pixel) {
        let index = self.index_of_pixel(x, y);
        self.data[index] = pixel;
    }

    /// Get the pixel value at the given coordinates
    ///
    /// # Panics
    /// This function panics if the coordinates are outside of the bitmap
    pub fn get_pixel(&self, x: usize, y: usize) -> Pixel {
        self.data[self.index_of_pixel(x, y)]
    }

    /// Calculate the index of the pixel data for a given set of coordinates
    fn index_of_pixel(&self, x: usize, y: usize) -> usize {
        debug_assert!(x < self.width);
        debug_assert!(y < self.height);

        y * self.width + x
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

    pub fn compose(&mut self, mask: Mask, source: Source, offset: Vec2D<usize>) {
        if offset.x < self.width && offset.y < self.height {
            // Don't draw out of bounds
            let available_space = Vec2D::new(self.width - offset.x, self.height - offset.y);
            match source {
                Source::Solid(color) => {
                    for x in 0..mask.width().min(available_space.x) {
                        for y in 0..mask.height().min(available_space.y) {
                            let opacity = mask.opacity_at(x, y).abs().min(1.);
                            let previous_color = self.get_pixel(x + offset.x, y + offset.y);
                            let computed_color = color.interpolate(Color(previous_color), opacity);
                            self.set_pixel(x + offset.x, y + offset.y, computed_color.into());
                        }
                    }
                },
            }
        }
    }
}
