use crate::ttf_tables::glyf::GlyphPoint;
use math::Vec2D;

#[derive(Clone, Copy, Debug)]
pub enum Operation {
    LineTo(Vec2D<i32>),
    QuadBezTo(Vec2D<i32>, Vec2D<i32>),
    MoveTo(Vec2D<i32>),
}

pub struct PathReader<I: Iterator<Item = GlyphPoint>> {
    inner: I,
    last_on_curve_point: Option<Vec2D<i32>>,
    previous_point: Option<GlyphPoint>,
    first_point_of_contour: Option<GlyphPoint>,
    state: PathReaderState,
}

impl<I: Iterator<Item = GlyphPoint>> PathReader<I> {
    pub fn new(inner: I) -> Self {
        Self {
            inner,
            previous_point: None,
            first_point_of_contour: None,
            state: PathReaderState::BeforeEndOfContour,
            last_on_curve_point: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PathReaderState {
    /// Regular operation
    BeforeEndOfContour,
    /// Indicates that after the path reader is done processing the current point,
    /// it should insert an extra curve back to the first point
    AtEndOfContour,
    /// Indicates that we previously closed a contour and should now start a new contour.
    AfterEndOfContour,
}

impl<I: Iterator<Item = GlyphPoint>> Iterator for PathReader<I> {
    type Item = Operation;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // If we previously connected the end points of two contours together,
            // then this is a new contour and we should discard the previous point
            if self.state == PathReaderState::AfterEndOfContour {
                self.state = PathReaderState::BeforeEndOfContour;
                self.previous_point = None;
            }

            // If we're at the end of a contour we don't consume a new point, instead
            // we connect the previous point with the first one in the contour
            let glyph_point = if self.state == PathReaderState::AtEndOfContour {
                self.state = PathReaderState::AfterEndOfContour;
                self.first_point_of_contour.unwrap()
            } else {
                let next_point = self.inner.next()?;

                if next_point.is_last_point_of_contour {
                    // If the point ends a contour, we set a marker. In this call to next(),
                    // we connect the previous point to the current point and in the next call,
                    // we will connect the current point to the first point of the contour (closing it)

                    // Ignore contours that only contain a single point
                    if self.first_point_of_contour.is_some() {
                        self.state = PathReaderState::AtEndOfContour;
                    }
                }
                next_point
            };

            match self.previous_point {
                Some(previous_point) => {
                    match (previous_point.is_on_curve, glyph_point.is_on_curve) {
                        (true, true) => {
                            // Consecutive on-curve points are connected by a line
                            self.previous_point = Some(glyph_point);
                            self.last_on_curve_point = Some(glyph_point.coordinates);
                            return Some(Operation::LineTo(glyph_point.coordinates));
                        },
                        (true, false) => {
                            // If the previous point was on the curve but the current one is not, this is the start of
                            // a bezier curve.
                            self.previous_point = Some(glyph_point);
                        },
                        (false, true) => {
                            // If the previous point was not on the curve but the current one is, we draw a bezier curve
                            self.previous_point = Some(glyph_point);
                            return Some(Operation::QuadBezTo(
                                previous_point.coordinates,
                                glyph_point.coordinates,
                            ));
                        },
                        (false, false) => {
                            // If multiple off-curve points occur consecutively, we insert a linearly interpolated mid point (on-curve) between them
                            let mid_point =
                                (previous_point.coordinates + glyph_point.coordinates) / 2;

                            self.last_on_curve_point = Some(mid_point);
                            self.previous_point = Some(glyph_point);
                            return Some(Operation::QuadBezTo(
                                previous_point.coordinates,
                                mid_point,
                            ));
                        },
                    }
                },
                None => {
                    // This is the start of a new contour, we move there
                    self.first_point_of_contour = Some(glyph_point);
                    self.previous_point = Some(glyph_point);
                    if glyph_point.is_on_curve {
                        self.last_on_curve_point = Some(glyph_point.coordinates);
                    }
                    return Some(Operation::MoveTo(glyph_point.coordinates));
                },
            }
        }
    }
}

pub trait PathConsumer {
    fn move_to(&mut self, p: Vec2D);
    fn line_to(&mut self, p: Vec2D);
    fn quad_bez_to(&mut self, p1: Vec2D, p2: Vec2D);
}
