#[derive(Clone, Copy, Debug)]
pub enum PixelFormat {
    GrayScale,
    RGB8,
    RGBA8,
}

#[derive(Clone)]
pub struct Image {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub format: PixelFormat,
}

impl Image {
    pub fn new(data: Vec<u8>, width: u32, height: u32, format: PixelFormat) -> Self {
        Self {
            data,
            width,
            height,
            format,
        }
    }

    pub fn resize(&self, new_width: usize, new_height: usize) -> Self {
        let new_data = vec![0; new_width * new_height * self.format.pixel_size()];

        let mut resized_image = Self {
            data: new_data,
            width: new_width as u32,
            height: new_height as u32,
            format: self.format,
        };

        // Find the closest pixel in the original image for each pixel in the new image and sample from that
        // (primitive resize algorithm but good enough for now)
        for x in 0..new_width {
            let closest_column =
                ((x as f32 / new_width as f32) * self.width as f32).round() as usize;
            for y in 0..new_height {
                let closest_row =
                    ((y as f32 / new_height as f32) * self.height as f32).round() as usize;
                resized_image
                    .pixel_at_mut(x, y)
                    .copy_from_slice(self.pixel_at(closest_column, closest_row));
            }
        }
        resized_image
    }

    pub fn pixel_at(&self, x: usize, y: usize) -> &[u8] {
        let pitch = self.width as usize * self.format.pixel_size();
        let pixel_is_at = pitch * y + x * self.format.pixel_size();
        &self.data[pixel_is_at..][..self.format.pixel_size()]
    }

    pub fn pixel_at_mut(&mut self, x: usize, y: usize) -> &mut [u8] {
        let pitch = self.width as usize * self.format.pixel_size();
        let pixel_is_at = pitch * y + x * self.format.pixel_size();
        &mut self.data[pixel_is_at..][..self.format.pixel_size()]
    }
}

impl PixelFormat {
    /// Return the number of bytes taken up by each pixel in the given format
    pub fn pixel_size(&self) -> usize {
        match &self {
            Self::GrayScale => 1,
            Self::RGB8 => 3,
            Self::RGBA8 => 4,
        }
    }
}
