use crate::ttf::tables::glyf::GlyphPoint;

#[derive(Clone, Copy, Debug)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct QuadraticBezier {
    pub p0: Point,
    pub p1: Point,
    pub p2: Point,
}

impl Point {
    pub fn scale(&mut self, scale: f32) {
        self.x *= scale;
        self.y *= scale;
    }
}

impl QuadraticBezier {
    pub fn scale(&mut self, scale: f32) {
        self.p0.scale(scale);
        self.p1.scale(scale);
        self.p2.scale(scale);
    }
}

impl From<GlyphPoint> for Point {
    fn from(value: GlyphPoint) -> Self {
        Self {
            x: value.coordinates.0 as f32,
            y: value.coordinates.1 as f32,
        }
    }
}

impl From<Point> for (usize, usize) {
    fn from(value: Point) -> Self {
        (value.x.round() as usize, value.y.round() as usize)
    }
}
