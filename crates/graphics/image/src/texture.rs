use std::{
    convert::FloatToInt,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use crate::{bmp, format::ColorFormat, jpeg, png, DynamicTexture, Rgba};

/// A texture that holds visual content
#[derive(Clone, Debug)]
pub struct Texture<C, D> {
    width: usize,
    height: usize,
    data: D,
    marker: PhantomData<C>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccessMode {
    Zero,
    Clamp,
}

impl<C, D> Texture<C, D> {
    #[must_use]
    pub const fn from_data(data: D, width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            data,
            marker: PhantomData,
        }
    }
}

impl<C> Texture<C, Vec<C::Channel>>
where
    C: ColorFormat,
    C::Channel: Copy + Default,
    f32: FloatToInt<C::Channel>,
{
    #[must_use]
    pub fn new(width: usize, height: usize) -> Self {
        Self::from_data(
            vec![C::Channel::default(); width * height * C::N_CHANNELS],
            width,
            height,
        )
    }

    /// Change the texture buffer size
    ///
    /// The contents of the texture are unspecified after this operation
    pub fn resize_buffer(&mut self, new_width: usize, new_height: usize) {
        self.width = new_width;
        self.height = new_height;
        self.data.resize(
            new_width * new_height * C::N_CHANNELS,
            C::Channel::default(),
        );
    }

    /// Constructs an empty texture
    #[must_use]
    pub fn empty() -> Self {
        Self::from_data(vec![], 0, 0)
    }
}

impl<'a, C, D> Texture<C, D>
where
    C: ColorFormat,
    C::Channel: 'a,
    D: 'a + Deref<Target = [C::Channel]>,
    f32: FloatToInt<C::Channel>,
{
    #[must_use]
    pub fn data(&self) -> &[C::Channel] {
        &self.data
    }

    #[must_use]
    pub fn pixel_data(&'a self, x: usize, y: usize) -> &[C::Channel] {
        let pixel_index = (self.width * y + x) * C::N_CHANNELS;
        &self.data[pixel_index..pixel_index + C::N_CHANNELS]
    }

    #[must_use]
    pub const fn width(&self) -> usize {
        self.width
    }

    #[must_use]
    pub const fn height(&self) -> usize {
        self.height
    }

    /// Return `true` if the coordinates are inside the bounds of the texutre
    #[must_use]
    pub const fn contains(&self, x: usize, y: usize) -> bool {
        x < self.width() && y < self.height()
    }

    /// Access a specific pixel in the image
    ///
    /// If the coordinates are outside the image, the nearest pixel in the
    /// image will be returned instead.
    ///
    /// # Panics
    /// Panics if any of the image dimensions is zero.
    #[must_use]
    pub fn get_clamped(&'a self, x: usize, y: usize) -> C {
        assert!(self.width() != 0);
        assert!(self.height() != 0);

        let clamped_x = x.min(self.width() - 1);
        let clamped_y = y.min(self.height() - 1);
        self.get_pixel(clamped_x, clamped_y)
    }

    #[must_use]
    pub fn get(&'a self, x: usize, y: usize, access_mode: AccessMode) -> C {
        match access_mode {
            AccessMode::Zero => self.get_or(x, y, C::default()),
            AccessMode::Clamp => self.get_clamped(x, y),
        }
    }

    /// Get the pixel value at the given coordinates
    ///
    /// # Panics
    /// This function panics if the coordinates are outside of the bitmap
    #[must_use]
    pub fn get_pixel(&'a self, x: usize, y: usize) -> C {
        C::from_channels(self.pixel_data(x, y))
    }

    /// Access a specific pixel in the image
    ///
    /// If the coordinates are outside the image, `default` will be returned
    #[must_use]
    pub fn get_or(&'a self, x: usize, y: usize, default: C) -> C {
        if self.contains(x, y) {
            self.get_pixel(x, y)
        } else {
            default
        }
    }
}

impl<C, D> Texture<C, D>
where
    C: ColorFormat,
    D: DerefMut<Target = [C::Channel]>,
    f32: FloatToInt<C::Channel>,
{
    #[must_use]
    pub fn data_mut(&mut self) -> &mut [C::Channel] {
        &mut self.data
    }

    #[must_use]
    pub fn pixel_data_mut(&mut self, x: usize, y: usize) -> &mut [C::Channel] {
        let pixel_index = (self.width() * y + x) * C::N_CHANNELS;
        &mut self.data[pixel_index..pixel_index + C::N_CHANNELS]
    }

    pub fn pixels_mut(&mut self) -> impl Iterator<Item = &mut [C::Channel]> {
        self.data.chunks_exact_mut(C::N_CHANNELS)
    }

    /// Set the pixel at the given coordinates to the specified value.
    ///
    /// # Panics
    /// This function panics if the coordinates are outside of the bitmap
    pub fn set_pixel(&mut self, x: usize, y: usize, pixel: C) {
        self.pixel_data_mut(x, y).copy_from_slice(pixel.channels());
    }

    pub fn clear(&mut self, clear_color: C) {
        let channels = clear_color.channels();
        for pixel in self.pixels_mut() {
            pixel.copy_from_slice(channels)
        }
    }
}

#[derive(Debug)]
pub enum Error {
    Bmp(bmp::Error),
    Png(png::Error),
    Jpeg(jpeg::Error),
}

impl DynamicTexture {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.starts_with(&png::PNG_HEADER) {
            Self::from_png(bytes).map_err(Error::from)
        } else if bytes.starts_with(&bmp::BMP_MAGIC) {
            Self::from_bmp(bytes).map_err(Error::from)
        } else {
            Self::from_jpeg(bytes).map_err(Error::from)
        }
    }

    pub fn from_bmp(bytes: &[u8]) -> Result<Self, bmp::Error> {
        bmp::decode(bytes)
    }

    pub fn from_jpeg(bytes: &[u8]) -> Result<Self, jpeg::Error> {
        jpeg::decode(bytes)
    }

    pub fn from_png(bytes: &[u8]) -> Result<Self, png::Error> {
        png::decode(bytes)
    }

    #[must_use]
    pub fn empty() -> Self {
        Self::Rgb8(Texture::empty())
    }

    #[must_use]
    pub const fn width(&self) -> usize {
        match self {
            Self::Rgb8(t) => t.width(),
            Self::Rgba8(t) => t.width(),
            Self::GrayScale8(t) => t.width(),
            Self::GrayScaleAlpha8(t) => t.width(),
        }
    }

    #[must_use]
    pub const fn height(&self) -> usize {
        match self {
            Self::Rgb8(t) => t.height(),
            Self::Rgba8(t) => t.height(),
            Self::GrayScale8(t) => t.height(),
            Self::GrayScaleAlpha8(t) => t.height(),
        }
    }

    #[must_use]
    pub fn get(&self, x: usize, y: usize, access_mode: AccessMode) -> Rgba<u8> {
        match self {
            Self::Rgb8(t) => t.get(x, y, access_mode).into(),
            Self::Rgba8(t) => t.get(x, y, access_mode),
            Self::GrayScale8(t) => t.get(x, y, access_mode).into(),
            Self::GrayScaleAlpha8(t) => t.get(x, y, access_mode).into(),
        }
    }
}

impl From<bmp::Error> for Error {
    fn from(value: bmp::Error) -> Self {
        Self::Bmp(value)
    }
}

impl From<png::Error> for Error {
    fn from(value: png::Error) -> Self {
        Self::Png(value)
    }
}

impl From<jpeg::Error> for Error {
    fn from(value: jpeg::Error) -> Self {
        Self::Jpeg(value)
    }
}
