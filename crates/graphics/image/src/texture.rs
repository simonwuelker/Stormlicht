use crate::png;

/// The target surface that content should be drawn to
#[derive(Clone, Debug)]
pub struct Texture<T> {
    width: usize,
    height: usize,
    data: Vec<T>,
}

#[derive(Clone, Copy, Debug)]
pub enum AccessMode<T> {
    Default(T),
    Clamp,
}

impl<T: Default + Copy> Texture<T> {
    #[must_use]
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            data: vec![T::default(); width * height],
        }
    }

    pub fn resize(&mut self, new_width: usize, new_height: usize) {
        self.width = new_width;
        self.height = new_height;
        self.data.resize(new_width * new_height, T::default());
    }
}

impl<T> Texture<T> {
    #[must_use]
    pub fn from_data(data: Vec<T>, width: usize, height: usize) -> Self {
        debug_assert_eq!(data.len(), width * height);

        Self {
            width,
            height,
            data,
        }
    }

    #[must_use]
    pub const fn width(&self) -> usize {
        self.width
    }

    #[must_use]
    pub const fn height(&self) -> usize {
        self.height
    }

    /// Set the pixel at the given coordinates to the specified value.
    ///
    /// # Panics
    /// This function panics if the coordinates are outside of the bitmap
    pub fn set_pixel(&mut self, x: usize, y: usize, pixel: T) {
        let index = self.index_of_pixel(x, y);
        self.data[index] = pixel;
    }

    /// Calculate the index of the pixel data for a given set of coordinates
    #[must_use]
    fn index_of_pixel(&self, x: usize, y: usize) -> usize {
        debug_assert!(self.contains(x, y));

        y * self.width + x
    }

    #[must_use]
    pub fn data(&self) -> &[T] {
        &self.data
    }

    /// Return `true` if the coordinates are inside the bounds of the texutre
    #[must_use]
    pub const fn contains(&self, x: usize, y: usize) -> bool {
        x < self.width() && y < self.height()
    }
}

impl<T: Copy> Texture<T> {
    pub fn clear(&mut self, clear_color: T) {
        self.data.fill(clear_color);
    }

    /// Get the pixel value at the given coordinates
    ///
    /// # Panics
    /// This function panics if the coordinates are outside of the bitmap
    #[must_use]
    pub fn get_pixel(&self, x: usize, y: usize) -> T {
        self.data[self.index_of_pixel(x, y)]
    }

    /// Access a specific pixel in the image
    ///
    /// If the coordinates are outside the image, `default` will be returned
    #[must_use]
    pub fn get_or(&self, x: usize, y: usize, default: T) -> T {
        if self.contains(x, y) {
            self.get_pixel(x, y)
        } else {
            default
        }
    }

    /// Access a specific pixel in the image
    ///
    /// If the coordinates are outside the image, the nearest pixel in the
    /// image will be returned instead.
    ///
    /// # Panics
    /// Panics if the image has no pixels (is empty)
    #[must_use]
    pub fn get_clamped(&self, x: usize, y: usize) -> T {
        let clamped_x = x.min(self.width() - 1);
        let clamped_y = y.min(self.height() - 1);
        self.get_pixel(clamped_x, clamped_y)
    }

    #[must_use]
    pub fn get(&self, x: usize, y: usize, access_mode: AccessMode<T>) -> T {
        match access_mode {
            AccessMode::Default(default) => self.get_or(x, y, default),
            AccessMode::Clamp => self.get_clamped(x, y),
        }
    }
}

impl Texture<u32> {
    pub fn from_png(bytes: &[u8]) -> Result<Self, png::Error> {
        png::decode(bytes)
    }
}
