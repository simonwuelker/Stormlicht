use std::fmt;

#[derive(Clone, Copy, Debug)]
pub struct Point {
    pub x: i16,
    pub y: i16,
}

impl Point {
    pub fn new(x: i16, y: i16) -> Self {
        Self { x: x, y: y }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BoundingBox {
    pub min_x: i16,
    pub max_x: i16,
    pub min_y: i16,
    pub max_y: i16,
}

impl BoundingBox {
    pub fn new(min_x: i16, min_y: i16, max_x: i16, max_y: i16) -> Self {
        Self {
            min_x: min_x,
            min_y: min_y,
            max_x: max_x,
            max_y: max_y,
        }
    }

    pub fn width(&self) -> i16 {
        self.max_x - self.min_x
    }

    pub fn height(&self) -> i16 {
        self.max_y - self.min_y
    }

    pub fn translate(&self, point: Point, into: Self) -> Point {
        Point {
            x: into.min_x
                + (((point.x - self.min_x) as f32 / (self.width() - 1) as f32)
                    * into.width() as f32) as i16,
            y: into.min_y
                + (((point.y - self.min_y) as f32 / (self.height() - 1) as f32)
                    * into.height() as f32) as i16,
        }
    }
}

pub struct Surface {
    width: usize,
    height: usize,
    data: Vec<u8>,
}

impl Surface {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width: width,
            height: height,
            data: vec![0; width * height],
        }
    }

    pub fn bounding_box(&self) -> BoundingBox {
        BoundingBox::new(0, 0, self.width as i16, self.height as i16)
    }

    fn is_point_on_map(&self, point: Vec2D) -> bool {
        point.x.is_sign_positive()
            && point.y.is_sign_positive()
            && (point.x.round() as usize) < self.width
            && (point.y.round() as usize) < self.height
    }

    pub fn set_pixel(&mut self, point: Vec2D, value: u8) {
        assert!(self.is_point_on_map(point));
        self.data[(self.height - point.y.round() as usize - 1) * self.height
            + point.x.round() as usize] = value;
    }

    pub fn line(&mut self, from: Vec2D, to: Vec2D) {
        // http://members.chello.at/~easyfilter/bresenham.html
        assert!(self.is_point_on_map(from));
        assert!(self.is_point_on_map(to));

        let delta_x = (to.x - from.x).abs();
        let step_x = if from.x < to.x { 1. } else { -1. };

        let delta_y = -(to.y - from.y).abs();
        let step_y = if from.y < to.y { 1. } else { -1. };
        let mut error = delta_x + delta_y;

        let mut current = from;
        loop {
            self.set_pixel(current, 255);
            if current == to {
                break;
            }

            let e2 = 2. * error;
            if e2 >= delta_y {
                error += delta_y;
                current.x += step_x;
            }
            if e2 <= delta_x {
                error += delta_x;
                current.y += step_y;
            }
        }
    }

    pub fn quad_bezier(&mut self, p0: Vec2D, p1: Vec2D, p2: Vec2D) {
        let arbitrary: f32 = 10.0;

        let delta_x = 2. * p1.x - p0.x - p2.x;
        let delta_y = 2. * p1.y - p0.y - p2.y;
        let total_delta = delta_x.powi(2) * delta_y.powi(2);

        if total_delta < arbitrary.recip() {
            self.line(p0, p2);
            return;
        }

        let num_segments = 1. + (arbitrary * total_delta).sqrt().floor();

        let mut t = 0.0;
        let step_size = num_segments.recip();
        let mut previous_point = p0.round();
        for _ in 0..num_segments as usize - 1 {
            t += step_size;
            let new_point = Vec2D::lerp(Vec2D::lerp(p0, p1, t), Vec2D::lerp(p1, p2, t), t).round();
            self.line(previous_point, new_point);
            previous_point = new_point;
        }
        // Draw the remainder of the curve
        self.line(previous_point, p2.round());
    }
}

impl fmt::Display for Surface {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        assert!(self.data.len() == self.height * self.width);

        for y in 0..self.height {
            for x in 0..self.width {
                let c = match self.data[y * self.width + x] {
                    0..=49 => '.',
                    50..=99 => '-',
                    100..=149 => '?',
                    150..=199 => 'X',
                    200.. => '#',
                };
                write!(f, "{}", c)?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

#[derive(PartialEq, Clone, Copy)]
pub struct Vec2D {
    x: f32,
    y: f32,
}

impl fmt::Debug for Vec2D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Vec 2D")
            .field(&self.x)
            .field(&self.y)
            .finish()
    }
}

impl Vec2D {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x: x, y: y }
    }

    pub fn lerp(a: Self, b: Self, t: f32) -> Self {
        let delta_x = b.x - a.x;
        let delta_y = b.y - a.y;

        Self {
            x: a.x + delta_x * t,
            y: a.y + delta_y * t,
        }
    }

    pub fn round(self) -> Self {
        Self {
            x: self.x.round(),
            y: self.y.round(),
        }
    }
}

pub trait RasterizerTarget {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn line(&mut self, from: Point, to: Point);
    fn bounding_box(&self) -> BoundingBox {
        BoundingBox::new(0, 0, self.width() as i16, self.height() as i16)
    }
}
