use crate::FlattenedPathPoint;
use math::{Rectangle, Vec2D};

#[derive(Clone, Debug)]
pub struct Rasterizer {
    width: usize,
    height: usize,
    offset: Vec2D,
    buffer: Vec<f32>,
}

impl Rasterizer {
    #[must_use]
    pub fn new(area: Rectangle<usize>, offset: Vec2D) -> Self {
        let width = area.width() + 1;
        let height = area.height() + 1;
        Self {
            width,
            height,
            offset,
            buffer: vec![0.; width * height],
        }
    }

    #[must_use]
    pub fn into_mask(self) -> Mask {
        Mask {
            width: self.width,
            height: self.height,
            mask: self.buffer,
        }
    }

    /// Rasterize a 2D Line.
    /// **Greatly** inspired by <https://github.com/raphlinus/font-rs/blob/master/src/raster.rs#L44>
    pub fn draw_line(&mut self, from: Vec2D, to: Vec2D) {
        // The rasterizer does not draw horizontal lines, those are covered by the fill
        // algorithm
        if (from.y - to.y).abs() <= f32::EPSILON {
            return;
        }

        // Make sure to always go from the lower point to the higher one
        let (direction, from, to) = if from.y < to.y {
            (1.0, from, to)
        } else {
            (-1.0, to, from)
        };

        let line_slope = (to.x - from.x) / (to.y - from.y);

        let mut x = from.x;
        let y_start = from.y as usize;
        if from.y.is_sign_negative() {
            x -= from.y * line_slope;
        }

        for y in y_start..self.height.min(to.y.ceil() as usize) {
            let linestart = y * self.width;

            // The y-delta covered by this line segment.
            // Will usually be zero, except for the first and last segments
            let dy = ((y + 1) as f32).min(to.y) - (y as f32).max(from.y);

            // The x coordinate where this line segment will end
            let xnext = x + line_slope * dy;

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

    pub fn fill(&mut self, path: &[FlattenedPathPoint]) {
        // Draw the outlines of the shape
        for line in path.array_windows::<2>() {
            if line[1].connected {
                self.draw_line(
                    line[0].coordinates - self.offset,
                    line[1].coordinates - self.offset,
                );
            }
        }

        self.fill_outline()
    }

    fn fill_outline(&mut self) {
        let mut accumulator = 0.;
        for elem in &mut self.buffer {
            accumulator += *elem;
            *elem = accumulator;
        }
    }
}

#[derive(Clone, Debug)]
pub struct Mask {
    width: usize,
    height: usize,
    mask: Vec<f32>,
}

impl Mask {
    #[must_use]
    pub fn width(&self) -> usize {
        self.width
    }

    #[must_use]
    pub fn height(&self) -> usize {
        self.height
    }

    #[must_use]
    pub fn opacity_at(&self, x: usize, y: usize) -> f32 {
        self.mask[y * self.width + x]
    }
}
