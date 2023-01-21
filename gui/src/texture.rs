//! A structure for pixelwise access to a canvas.
//!
//! Note that unlike the [sdl2::render::Texture], a [Texture] is not computed on the GPU
//! and is not related to the former.

use crate::GuiError;
use anyhow::Result;
use image::{Image, PixelFormat};
pub use sdl2::pixels::PixelFormatEnum as SDLPixelFormat;
use sdl2::surface::Surface;

pub struct Texture {
    data: Vec<u8>,
    width: u32,
    height: u32,
    format: SDLPixelFormat,
}

impl Texture {
    pub fn new(data: Vec<u8>, width: u32, height: u32, format: SDLPixelFormat) -> Self {
        Self {
            data,
            width,
            height,
            format,
        }
    }

    pub fn as_surface(&mut self) -> Result<Surface> {
        let surface = Surface::from_data(&mut self.data, self.width, self.height, 0, self.format)
            .map_err(GuiError::from_sdl)?;
        Ok(surface)
    }
}

impl From<Image> for Texture {
    fn from(value: Image) -> Self {
        let format = match value.format {
            PixelFormat::RGB8 => SDLPixelFormat::RGB888,
            PixelFormat::RGBA8 => SDLPixelFormat::RGBA8888,
            _ => todo!(),
        };

        Self {
            data: value.data,
            width: value.width,
            height: value.height,
            format: format,
        }
    }
}
