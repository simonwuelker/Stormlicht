#[derive(Clone, Copy, Debug, Default)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    /// Linearly interpolate between two points.
    /// The parameter `t` specifies the weight between the two points
    /// and should generally be in `[0, 1]`.
    /// If `t` is `0`, the result will be `p0`.
    /// If `t` is `1`, the result will be `p1`.
    /// Otherwise, the result will be a point on the line between `p0` and `p1`.
    pub fn lerp(p0: Self, p1: Self, t: f32) -> Self {
        Self {
            x: p0.x + t * (p1.x - p0.x),
            y: p0.y + t * (p1.y - p0.y),
        }
    }
}

pub struct Rasterizer {
    width: usize,
    height: usize,
    buffer: Vec<f32>,
}

impl Rasterizer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            buffer: vec![0.0; width * height],
        }
    }

    pub fn clear(&mut self) {
        for i in self.buffer.iter_mut() {
            *i = 0.0;
        }
    }

    /// Rasterize a 2D Line.
    /// **Greatly** inspired by <https://github.com/raphlinus/font-rs/blob/master/src/raster.rs#L44>
    pub fn draw_line(&mut self, from: Point, to: Point) {
        // println!("Line from {from:?} to {to:?}");
        if (from.y - to.y).abs() <= f32::EPSILON {
            return;
        }

        let (direction, from, to) = if from.y < to.y {
            (1.0, from, to)
        } else {
            (-1.0, to, from)
        };

        let dxdy = (to.x - from.x) / (to.y - from.y);
        let mut x = from.x;
        let y_start = from.y as usize;
        if from.y.is_sign_negative() {
            x -= from.y * dxdy;
        }

        for y in y_start..self.height.min(to.y.ceil() as usize) {
            let linestart = y * self.width;
            let dy = ((y + 1) as f32).min(to.y) - (y as f32).max(from.y);
            let xnext = x + dxdy * dy;
            let d = dy * direction;
            let (x0, x1) = if x < xnext { (x, xnext) } else { (xnext, x) };
            let x0floor = x0.floor();
            let x0i = x0floor as i32;
            let x1ceil = x1.ceil();
            let x1i = x1ceil as i32;
            if x1i <= x0i + 1 {
                let xmf = 0.5 * (x + xnext) - x0floor;
                let linestart_x0i = linestart as isize + x0i as isize;
                if linestart_x0i.is_negative() {
                    continue;
                }
                self.buffer[linestart_x0i as usize] += d - d * xmf;
                self.buffer[linestart_x0i as usize] += d * xmf;
            } else {
                let s = (x1 - x0).recip();
                let x0f = x0 - x0floor;
                let a0 = 0.5 * s * (1.0 - x0f) * (1.0 - x0f);
                let x1f = x1 - x1ceil + 1.0;
                let am = 0.5 * s * x1f * x1f;
                let linestart_x0i = linestart as isize + x0i as isize;
                if linestart_x0i.is_negative() {
                    continue;
                }
                self.buffer[linestart_x0i as usize + 1] += d * a0;
                if x1i == x0i + 2 {
                    self.buffer[linestart_x0i as usize + 1] += d * (1.0 - a0 - am);
                } else {
                    let a1 = s * (1.5 - x0f);
                    self.buffer[linestart_x0i as usize + 1] += d * (a1 - a0);
                    for xi in x0i + 2..x1i - 1 {
                        self.buffer[linestart + xi as usize] += d * s;
                    }
                    let a2 = a1 + (x1i - x0i - 3) as f32 * s;
                    self.buffer[linestart + (x1i - 1) as usize] += d * (1.0 - a2 - am);
                }

                self.buffer[linestart + x1i as usize] += d * am;
            }
            x = xnext;
        }
    }

    /// Rasterize a quadratic Bézier curve
    /// **Greatly** inspired by <https://github.com/raphlinus/font-rs/blob/master/src/raster.rs#L106>
    pub fn draw_quad_bezier(&mut self, p0: Point, p1: Point, p2: Point) {
        // println!("bezier from {p0:?} over {p1:?} to {p2:?}");
        let dev_x = p0.x - 2.0 * p1.x + p2.x;
        let dev_y = p0.y - 2.0 * p1.y + p2.y;
        let devsq = dev_x * dev_x + dev_y * dev_y;
        if devsq < 0.333 {
            // The control point is so close to the direct line
            // that drawing a straight line suffices
            self.draw_line(p0, p2);
        }

        let tol = 3.0;

        // Approximate the number of segments that we should interpolate the
        // curve into
        let n = 1 + (tol * devsq).sqrt().sqrt().floor() as usize;

        let mut p = p0;
        let nrecip = (n as f32).recip();
        let mut t = 0.0;
        for _ in 0..n {
            t += nrecip;
            let pn = Point::lerp(Point::lerp(p0, p1, t), Point::lerp(p1, p2, t), t);
            self.draw_line(p, pn);
            p = pn;
        }
    }
    pub fn for_each_pixel<F: FnMut((usize, usize), u8)>(&self, mut callback: F) {
        let mut accumulator: f32 = 0.;
        for y in 0..self.height {
            for x in 0..self.width {
                let i = y * self.width + x;
                accumulator += self.buffer[i];
                callback((x, y), (accumulator.abs().min(1.0) * 255.0) as u8);
            }
        }
    }
}
