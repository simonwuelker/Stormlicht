use crate::PixelFormat;

/// A drawable surface for generic graphic rendering.
pub struct Canvas {
    width: usize,
    height: usize,
    data: Vec<u8>,
    format: PixelFormat,
}

impl Canvas {
    pub fn new(data: Vec<u8>, width: usize, height: usize, format: PixelFormat) -> Self {
        Self {
            width: width,
            height: height,
            data: data,
            format: format,
        }
    }

    pub fn new_uninit(width: usize, height: usize, format: PixelFormat) -> Self {
        Self::new(
            vec![0; width * height * format.pixel_size()],
            width,
            height,
            format,
        )
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn format(&self) -> PixelFormat {
        self.format
    }

    pub fn resize(&self, new_width: usize, new_height: usize) -> Self {
        let new_data = vec![0; new_width * new_height * self.format.pixel_size()];

        let mut resized_image = Self {
            data: new_data,
            width: new_width,
            height: new_height,
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
        assert!(self.contains_point(x, y));

        let pitch = self.width * self.format.pixel_size();
        let pixel_is_at = pitch * y + x * self.format.pixel_size();
        &self.data[pixel_is_at..][..self.format.pixel_size()]
    }

    pub fn pixel_at_mut(&mut self, x: usize, y: usize) -> &mut [u8] {
        assert!(self.contains_point(x, y));

        let pitch = self.width * self.format.pixel_size();
        let pixel_is_at = pitch * y + x * self.format.pixel_size();
        &mut self.data[pixel_is_at..][..self.format.pixel_size()]
    }

    pub fn contains_point(&self, x: usize, y: usize) -> bool {
        x < self.width() && y < self.height()
    }

    pub fn line(&mut self, from: (usize, usize), to: (usize, usize), color: &[u8]) {
        // http://members.chello.at/~easyfilter/bresenham.html
        // assert!(self.contains_point(from.0, from.1));
        // assert!(self.contains_point(to.0, to.1));

        // Cast to signed numbers because they are easier to work with
        let from_x = from.0 as isize;
        let from_y = from.1 as isize;
        let to_x = to.0 as isize;
        let to_y = to.1 as isize;

        let delta_x = (to_x - from_x).abs();
        let step_x = if from_x < to_x { 1 } else { -1 };

        let delta_y = -(to_y - from_y).abs();
        let step_y: isize = if from_y < to_y { 1 } else { -1 };

        let mut error = delta_x + delta_y;
        let mut current = from;
        loop {
            if self.contains_point(current.0, current.1) {
                self.pixel_at_mut(current.0, current.1)
                    .copy_from_slice(color);
            }

            if current == to {
                break;
            }

            let e2 = 2 * error;
            if e2 >= delta_y {
                error += delta_y;
                current.0 = current.0.wrapping_add_signed(step_x);
            }
            if e2 <= delta_x {
                error += delta_x;
                current.1 = current.1.wrapping_add_signed(step_y);
            }
        }
    }

    pub fn quad_bezier(
        &mut self,
        p0: (usize, usize),
        _p1: (usize, usize),
        p2: (usize, usize),
        color: &[u8],
    ) {
        self.line(p0, p2, color);
        // let arbitrary: f32 = 10.0;

        // let delta_x = 2. * p1.x - p0.x - p2.x;
        // let delta_y = 2. * p1.y - p0.y - p2.y;
        // let total_delta = delta_x.powi(2) * delta_y.powi(2);

        // if total_delta < arbitrary.recip() {
        //     self.line(p0, p2);
        //     return;
        // }

        // let num_segments = 1. + (arbitrary * total_delta).sqrt().floor();

        // let mut t = 0.0;
        // let step_size = num_segments.recip();
        // let mut previous_point = p0.round();
        // for _ in 0..num_segments as usize - 1 {
        //     t += step_size;
        //     let new_point = Vec2D::lerp(Vec2D::lerp(p0, p1, t), Vec2D::lerp(p1, p2, t), t).round();
        //     self.line(previous_point, new_point);
        //     previous_point = new_point;
        // }
        // // Draw the remainder of the curve
        // self.line(previous_point, p2.round());
    }
}
