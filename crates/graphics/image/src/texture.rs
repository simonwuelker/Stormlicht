use crate::{bmp, jpeg, png};

#[derive(Clone, Copy, Debug, Default)]
pub struct Rgbaf32 {
    channels: [f32; 4],
}

impl Rgbaf32 {
    pub const BLANK: Self = Self::rgba(0., 0., 0., 0.);

    #[inline]
    #[must_use]
    pub const fn red(&self) -> f32 {
        self.channels[0]
    }

    #[inline]
    #[must_use]
    pub const fn green(&self) -> f32 {
        self.channels[1]
    }

    #[inline]
    #[must_use]
    pub const fn blue(&self) -> f32 {
        self.channels[2]
    }

    #[inline]
    #[must_use]
    pub const fn alpha(&self) -> f32 {
        self.channels[3]
    }

    #[inline]
    #[must_use]
    pub const fn grayscale(value: f32) -> Self {
        Self::grayscale_with_alpha(value, 1.)
    }

    #[inline]
    #[must_use]
    pub const fn grayscale_with_alpha(value: f32, alpha: f32) -> Self {
        Self {
            channels: [value, value, value, alpha],
        }
    }

    #[inline]
    #[must_use]
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self::rgba(r, g, b, 1.)
    }

    #[inline]
    #[must_use]
    pub const fn rgba(r: f32, g: f32, b: f32, alpha: f32) -> Self {
        Self {
            channels: [r, g, b, alpha],
        }
    }

    #[inline]
    pub fn set_alpha(&mut self, alpha: f32) {
        self.channels[3] = alpha;
    }

    /// Blend another color on top of `self`
    #[must_use]
    pub fn blend(&self, other: Self) -> Self {
        // https://stackoverflow.com/questions/7438263/alpha-compositing-algorithm-blend-modes#answer-11163848
        if other.alpha() == 0. {
            return *self;
        }

        if other.alpha() == 1. {
            return other;
        }

        let new_alpha = self.alpha() + other.alpha() - self.alpha() * other.alpha();

        if new_alpha == 0. {
            // New image doesn't have any color
            return Self::BLANK;
        }

        let red = other.red() * other.alpha() + self.red() * (1. - other.alpha());
        let green = other.green() * other.alpha() + self.green() * (1. - other.alpha());
        let blue = other.blue() * other.alpha() + self.blue() * (1. - other.alpha());

        let channels = [red, green, blue, new_alpha];

        Self { channels }
    }
}
/// A texture that holds visual content
#[derive(Clone, Debug)]
pub struct Texture {
    width: usize,
    height: usize,
    data: Vec<Rgbaf32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccessMode {
    Zero,
    Clamp,
}

impl Texture {
    #[must_use]
    pub const fn from_data(data: Vec<Rgbaf32>, width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            data,
        }
    }
}

impl Texture {
    #[must_use]
    pub fn new(width: usize, height: usize) -> Self {
        Self::from_data(vec![Rgbaf32::default(); width * height], width, height)
    }

    /// Change the texture buffer size
    ///
    /// The contents of the texture are unspecified after this operation
    pub fn resize_buffer(&mut self, new_width: usize, new_height: usize) {
        self.width = new_width;
        self.height = new_height;
        self.data.resize(new_width * new_height, Rgbaf32::default());
    }

    /// Constructs an empty texture
    #[must_use]
    pub fn empty() -> Self {
        Self::from_data(vec![], 0, 0)
    }

    pub fn resize(&self, width: usize, height: usize) -> Self {
        let mut result = Self::new(width, height);

        if self.width() == 0 || self.height() == 0 {
            return result;
        }

        // Nearest neighbor
        let height_ratio = self.height as f32 / height as f32;
        let width_ratio = self.width as f32 / width as f32;

        for y in 0..height {
            let nearest_y = (y as f32 * height_ratio).floor() as usize;
            for x in 0..width {
                let nearest_x = (x as f32 * width_ratio).floor() as usize;
                result.set_pixel(x, y, self.get_pixel(nearest_x, nearest_y));
            }
        }

        result
    }
}

impl Texture {
    #[must_use]
    pub fn data(&self) -> &[Rgbaf32] {
        &self.data
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
    pub fn get_clamped(&self, x: usize, y: usize) -> Rgbaf32 {
        assert!(self.width() != 0);
        assert!(self.height() != 0);

        let clamped_x = x.min(self.width() - 1);
        let clamped_y = y.min(self.height() - 1);
        self.get_pixel(clamped_x, clamped_y)
    }

    #[must_use]
    pub fn get(&self, x: usize, y: usize, access_mode: AccessMode) -> Rgbaf32 {
        match access_mode {
            AccessMode::Zero => self.get_or(x, y, Rgbaf32::default()),
            AccessMode::Clamp => self.get_clamped(x, y),
        }
    }

    /// Get the pixel value at the given coordinates
    ///
    /// # Panics
    /// This function panics if the coordinates are outside of the bitmap
    #[must_use]
    pub fn get_pixel(&self, x: usize, y: usize) -> Rgbaf32 {
        let pixel_index = self.width * y + x;
        self.data[pixel_index]
    }

    /// Access a specific pixel in the image
    ///
    /// If the coordinates are outside the image, `default` will be returned
    #[must_use]
    pub fn get_or(&self, x: usize, y: usize, default: Rgbaf32) -> Rgbaf32 {
        if self.contains(x, y) {
            self.get_pixel(x, y)
        } else {
            default
        }
    }
}

impl Texture {
    #[must_use]
    pub fn data_mut(&mut self) -> &mut [Rgbaf32] {
        &mut self.data
    }

    #[must_use]
    pub fn pixel_data_mut(&mut self, x: usize, y: usize) -> &mut Rgbaf32 {
        let pixel_index = self.width() * y + x;
        &mut self.data[pixel_index]
    }

    pub fn pixels_mut(&mut self) -> impl Iterator<Item = &mut Rgbaf32> {
        self.data.iter_mut()
    }

    /// Set the pixel at the given coordinates to the specified value.
    ///
    /// # Panics
    /// This function panics if the coordinates are outside of the bitmap
    pub fn set_pixel(&mut self, x: usize, y: usize, pixel: Rgbaf32) {
        *self.pixel_data_mut(x, y) = pixel;
    }

    pub fn clear(&mut self, clear_color: Rgbaf32) {
        for pixel in self.pixels_mut() {
            *pixel = clear_color
        }
    }
}

#[derive(Debug)]
pub enum Error {
    Bmp(bmp::Error),
    Png(png::Error),
    Jpeg(jpeg::Error),
}

impl Texture {
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
