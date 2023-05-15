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

    pub fn set_pixel(&mut self, x: usize, y: usize, pixel: Pixel) {
        let index = self.index_of_pixel(x, y);
        self.data[index] = pixel;
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> Pixel {
        self.data[self.index_of_pixel(x, y)]
    }

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
        // log::info!(
        //     "Composing mask of size {}x{} at {offset:?}",
        //     mask.width(),
        //     mask.height()
        // );
        match source {
            Source::Solid(color) => {
                for x in 0..mask.width() {
                    for y in 0..mask.height() {
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
