use crate::{vec2d::Angle, Vec2D};

#[derive(Clone, Copy, Debug)]
pub struct Spline {
    pub curvature: Angle,
    /// Start point
    pub p0: Vec2D,
    /// End point
    pub p2: Vec2D,
}

#[derive(Clone, Copy, Debug)]
pub enum PathCommand {
    MoveTo(Vec2D),
    LineTo(Vec2D),
    QuadTo(Vec2D, Vec2D),
}

#[derive(Clone, Debug)]
pub struct Path {
    start: Vec2D,
    commands: Vec<PathCommand>,
}

/// Approximates the number of segments required to
impl Path {
    pub fn new(start: Vec2D) -> Self {
        Self {
            start,
            commands: vec![],
        }
    }

    /// Move the write head to a new point without creating a connecting [Spline].
    pub fn move_to(mut self, to: Vec2D) -> Self {
        match self.commands.last_mut() {
            Some(PathCommand::MoveTo(point)) => {
                *point = to;
            },
            Some(_) => self.commands.push(PathCommand::MoveTo(to)),
            None => self.start = to,
        }

        self
    }

    /// Create a straight [Spline] from the current position to the point.
    pub fn line(mut self, point: Vec2D) -> Self {
        self.commands.push(PathCommand::LineTo(point));
        self
    }

    pub fn quad_bez_to(mut self, p1: Vec2D, p2: Vec2D) -> Self {
        self.commands.push(PathCommand::QuadTo(p1, p2));
        self
    }

    /// Flatten quadratic bezier curves as described by [Raph Levien on his blog](https://raphlinus.github.io/graphics/curves/2019/12/23/flatten-quadbez.html)
    pub fn flatten(&self, tolerance: f32) -> FlattenedPath {
        let mut flattened_path = FlattenedPath { lines: vec![] };
        let sqrt_tolerance = tolerance.sqrt();

        let mut current_point = self.start;
        let mut start_new_contour = true;
        for &command in &self.commands {
            match command {
                PathCommand::MoveTo(point) => {
                    current_point = point;
                    start_new_contour = true;
                },
                PathCommand::LineTo(point) => {
                    flattened_path.lines.push((point, start_new_contour));
                    start_new_contour = false;
                },
                PathCommand::QuadTo(p1, p2) => {
                    let curve = QuadraticBezier {
                        p0: current_point,
                        p1,
                        p2,
                    };
                    let segment_parameters =
                        curve.approximate_number_of_segments_required(sqrt_tolerance);
                    let n_subdivisions =
                        ((0.5 * segment_parameters.val / sqrt_tolerance).ceil() as usize).max(1);
                    let step_size = (n_subdivisions as f32).recip();
                    for i in 1..n_subdivisions {
                        let progress = i as f32 * step_size;
                        let t = segment_parameters.determine_subdiv_t(progress);
                        let curve_value_at_t = curve.evaluate_at(t);
                        flattened_path.lines.push((curve_value_at_t, false));
                    }

                    // Connect to the end of the contour
                    flattened_path.lines.push((p2, false));

                    start_new_contour = false;
                },
            }
        }

        flattened_path
    }
}

#[inline]
fn approximate_parabola_integral(x: f32) -> f32 {
    const D: f32 = 0.64;
    x / (1.0 - D + (D.powi(4) + 0.25 * x * x).sqrt().sqrt())
}

#[inline]
fn approximate_inverse_parabola_integral(x: f32) -> f32 {
    const B: f32 = 0.39;
    x * (1.0 - B + (B * B + 0.25 * x * x).sqrt())
}

#[derive(Clone, Copy, Debug)]
struct QuadraticBezier {
    p0: Vec2D,
    p1: Vec2D,
    p2: Vec2D,
}

impl QuadraticBezier {
    /// <https://github.com/linebender/kurbo/blob/master/src/quadbez.rs#L57-L93>
    fn approximate_number_of_segments_required(
        &self,
        sqrt_tolerance: f32,
    ) -> CurveFlattenParameters {
        // Compute the parameters to transform the bezier curve to a regular parabola.
        let d01 = self.p1 - self.p0;
        let d12 = self.p2 - self.p1;
        let dd = d01 - d12;
        let cross = (self.p2 - self.p0).cross_product(dd);
        let x0 = d01.dot(dd) * cross.recip();
        let x2 = d12.dot(dd) * cross.recip();
        let scale = (cross / (dd.magnitude() * (x2 - x0))).abs();

        // Compute number of required subdivisions
        let a0 = approximate_parabola_integral(x0);
        let a2 = approximate_parabola_integral(x2);
        let val = if scale.is_finite() {
            let delta_a = (a2 - a0).abs();
            let sqrt_scale = scale.sqrt();
            if x0.signum() == x2.signum() {
                delta_a * sqrt_scale
            } else {
                let xmin = sqrt_tolerance / sqrt_scale;
                sqrt_tolerance * delta_a / approximate_parabola_integral(xmin)
            }
        } else {
            0.
        };
        let u0 = approximate_inverse_parabola_integral(a0);
        let u2 = approximate_inverse_parabola_integral(a2);
        let u_scale = (u2 - u0).recip();

        CurveFlattenParameters {
            a0,
            a2,
            u0,
            u_scale,
            val,
        }
    }

    fn evaluate_at(&self, t: f32) -> Vec2D {
        let mt = 1.0 - t;
        self.p0 * (mt * mt) + (self.p1 * (mt * 2.0) + self.p2 * t) * t
    }
}

#[derive(Clone, Copy, Debug)]
struct CurveFlattenParameters {
    a0: f32,
    a2: f32,
    u0: f32,
    u_scale: f32,
    val: f32,
}

impl CurveFlattenParameters {
    fn determine_subdiv_t(&self, x: f32) -> f32 {
        let a = self.a0 + (self.a2 - self.a0) * x;
        let u = approximate_inverse_parabola_integral(a);
        (u - self.u0) * self.u_scale
    }
}

/// Similar to [Path], except all the curves have been flattened to straight lines.
/// Obtained by calling [Path::flatten].
#[derive(Clone, Debug)]
pub struct FlattenedPath {
    /// The lines that make up the [Path]
    ///
    /// Each point is associated with a boolean describing whether or not this point starts
    /// a new contour
    lines: Vec<(Vec2D, bool)>,
}
