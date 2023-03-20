use crate::ttf_tables::glyf::GlyphPoint;

#[derive(Clone, Copy, Debug)]
pub enum Operation {
    LineTo(DiscretePoint),
    QuadBezTo(DiscretePoint, DiscretePoint),
    MoveTo(DiscretePoint),
}

#[derive(Clone, Copy, Debug)]
pub struct DiscretePoint {
    pub x: i16,
    pub y: i16,
}

impl DiscretePoint {
    pub fn mid(p0: Self, p1: Self) -> Self {
        Self {
            x: (p0.x + p1.x) / 2,
            y: (p0.y + p1.y) / 2,
        }
    }
}

impl From<GlyphPoint> for DiscretePoint {
    fn from(value: GlyphPoint) -> Self {
        Self {
            x: value.coordinates.0,
            y: value.coordinates.1,
        }
    }
}

pub struct PathReader<I: Iterator<Item = GlyphPoint>> {
    inner: I,
    last_on_curve_point: Option<DiscretePoint>,
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
    BeforeEndOfContour,
    AtEndOfContour,
    AfterEndOfContour,
}

impl<I: Iterator<Item = GlyphPoint>> Iterator for PathReader<I> {
    type Item = Operation;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // If we previously connected the end points of two contours together,
            // we should discard any previous points
            if self.state == PathReaderState::AfterEndOfContour {
                self.state = PathReaderState::BeforeEndOfContour;
                self.previous_point = None;
            }

            let glyph_point = if self.state == PathReaderState::AtEndOfContour {
                self.state = PathReaderState::AfterEndOfContour;
                self.first_point_of_contour.unwrap()
            } else {
                self.inner.next()?
            };

            if glyph_point.is_last_point_of_contour {
                // If the point ends a contour, we set a marker. In this call to next(),
                // we connect the previous point to the current point and in the next call,
                // we will connect the current point to the first point of the contour (closing it)

                // Ignore contours that only contain a single point
                if self.first_point_of_contour.is_some() {
                    self.state = PathReaderState::AtEndOfContour;
                }
            }

            match self.previous_point {
                Some(previous_point) => {
                    match (previous_point.is_on_curve, glyph_point.is_on_curve) {
                        (true, true) => {
                            // Consecutive on-curve points are connected by a line
                            self.previous_point = Some(glyph_point);
                            self.last_on_curve_point = Some(glyph_point.into());
                            return Some(Operation::LineTo(glyph_point.into()));
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
                                previous_point.into(),
                                glyph_point.into(),
                            ));
                        },
                        (false, false) => {
                            // If multiple off-curve points occur consecutively, we insert a linearly interpolated mid point (on-curve) between them
                            let mid_point =
                                DiscretePoint::mid(previous_point.into(), glyph_point.into());
                            self.last_on_curve_point = Some(mid_point);
                            self.previous_point = Some(glyph_point);
                            return Some(Operation::QuadBezTo(previous_point.into(), mid_point));
                        },
                    }
                },
                None => {
                    // This is the start of a new contour, we move there
                    self.first_point_of_contour = Some(glyph_point);
                    self.previous_point = Some(glyph_point);
                    if glyph_point.is_on_curve {
                        self.last_on_curve_point = Some(glyph_point.into());
                    }
                    return Some(Operation::MoveTo(glyph_point.into()));
                },
            }
        }
    }
}
