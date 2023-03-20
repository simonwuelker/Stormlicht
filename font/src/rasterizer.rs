#[derive(Clone, Copy, Debug)]
pub struct Point {
    pub x: f32,
    pub y: f32,
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

    /// Rasterize a 2D Line.
    /// **Greatly** inspired by <https://github.com/raphlinus/font-rs/blob/master/src/raster.rs#L44>
    pub fn draw_line(&mut self, from: Point, to: Point) {
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
