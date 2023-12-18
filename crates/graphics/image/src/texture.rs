/// The target surface that content should be drawn to
#[derive(Clone, Debug)]
pub struct Texture<T> {
    width: usize,
    height: usize,
    data: Vec<T>,
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
}
