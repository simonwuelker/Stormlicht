/// The target surface that content should be drawn to
#[derive(Clone, Debug)]
pub struct Bitmap<T: Copy> {
    width: usize,
    height: usize,
    data: Vec<T>,
}

impl<T: Default + Copy> Bitmap<T> {
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

impl<T: Copy> Bitmap<T> {
    pub fn from_data(data: Vec<T>, width: usize, height: usize) -> Self {
        debug_assert_eq!(data.len(), width * height);

        Self {
            width,
            height,
            data,
        }
    }
    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
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

    /// Get the pixel value at the given coordinates
    ///
    /// # Panics
    /// This function panics if the coordinates are outside of the bitmap
    pub fn get_pixel(&self, x: usize, y: usize) -> T {
        self.data[self.index_of_pixel(x, y)]
    }

    /// Calculate the index of the pixel data for a given set of coordinates
    fn index_of_pixel(&self, x: usize, y: usize) -> usize {
        debug_assert!(x < self.width);
        debug_assert!(y < self.height);

        y * self.width + x
    }

    pub fn data(&self) -> &[T] {
        &self.data
    }

    pub fn clear(&mut self, clear_color: T) {
        self.data.fill(clear_color);
    }
}
