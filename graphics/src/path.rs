use crate::{point::Angle, Point};

#[derive(Clone, Copy, Debug)]
pub struct Spline {
    pub curvature: Angle,
    /// Start point
    pub p0: Point,
    /// End point
    pub p2: Point,
}

#[derive(Clone, Debug)]
pub struct Path {
    /// The last point of the previous spline
    current_position: Point,

    /// The splines that make up the [Path]
    splines: Vec<Spline>,

    /// Remembers the outgoing angle of the last spline to potentially
    /// join the next spline with the previous one, if their angles are similar.
    /// `None` if this is the first spline in the current contour.
    /// Note that this is not equal to the last [Spline]s `curvature`!
    angle_of_last_spline: Option<Angle>,
}

impl Path {
    pub fn new(start: Point) -> Self {
        Self {
            current_position: start,
            splines: vec![],
            angle_of_last_spline: None,
        }
    }

    /// Move the write head to a new point without creating a connecting [Spline].
    pub fn move_to(mut self, to: Point) -> Self {
        self.angle_of_last_spline = None;
        self.current_position = to;
        self
    }

    /// Create a straight [Spline] from the current position to the point.
    pub fn line(mut self, point: Point) -> Self {
        let direction = point - self.current_position;
        self.angle_of_last_spline = Some(direction.angle());
        self.splines.push(Spline {
            curvature: Angle::from_radians(0.0),
            p0: self.current_position,
            p2: point,
        });
        self.current_position = point;
        self
    }

    pub fn quad_bez_to(self, p1: Point, p2: Point) -> Self {
        _ = p1;
        _ = p2;
        todo!()
    }
}
