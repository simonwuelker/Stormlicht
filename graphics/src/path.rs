use crate::math::Vec2D;

#[derive(Clone, Copy, Debug)]
pub enum PathCommand {
    MoveTo(Vec2D),
    LineTo(Vec2D),
    QuadTo(Vec2D, Vec2D),
}

#[derive(Clone, Debug)]
pub struct Path {
    /// The bounding rectangle of the paths contours.
    /// This might be larger than the actual path, but will never be smaller.
    // extents: Rectangle,
    start: Vec2D,
    commands: Vec<PathCommand>,
}

/// Approximates the number of segments required to
impl Path {
    pub fn empty() -> Self {
        Self {
            // extents: Rectangle::default(),
            start: Vec2D::default(),
            commands: vec![],
        }
    }

    pub fn new(start: Vec2D) -> Self {
        Self {
            // extents: Rectangle::default(),
            start,
            commands: vec![],
        }
    }

    // Close the current contour
    pub fn close_contour(mut self) -> Self {
        if !self.commands.is_empty() {
            self.commands.push(PathCommand::LineTo(self.start));
        }
        self
    }

    /// Move the write head to a new point without connecting the two.
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

    /// Create a straight Line from the current position to the point.
    pub fn line_to(mut self, point: Vec2D) -> Self {
        self.commands.push(PathCommand::LineTo(point));
        self
    }

    pub fn quad_bez_to(mut self, p1: Vec2D, p2: Vec2D) -> Self {
        self.commands.push(PathCommand::QuadTo(p1, p2));
        self
    }

    /// Flatten quadratic bezier curves as described by [Raph Levien](https://raphlinus.github.io/graphics/curves/2019/12/23/flatten-quadbez.html)
    ///
    /// # Visualization
    /// <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 120 30" height="30mm" width="120mm">
    ///   <path d="M26.7 24.94l.82-11.15M44.46 5.1L33.8 7.34" fill="none" stroke="#55d400" stroke-width=".5"/>
    ///   <path d="M26.7 24.94c.97-11.13 7.17-17.6 17.76-19.84M75.27 24.94l1.13-5.5 2.67-5.48 4-4.42L88 6.7l5.02-1.6" fill="none" stroke="#000"/>
    ///   <path d="M77.57 19.37a1.1 1.1 0 0 1-1.08 1.08 1.1 1.1 0 0 1-1.1-1.08 1.1 1.1 0 0 1 1.08-1.1 1.1 1.1 0 0 1 1.1 1.1" color="#000" fill="none" stroke="#030303" stroke-linecap="round" stroke-opacity=".5"/>
    ///   <path d="M77.57 19.37a1.1 1.1 0 0 1-1.08 1.08 1.1 1.1 0 0 1-1.1-1.08 1.1 1.1 0 0 1 1.08-1.1 1.1 1.1 0 0 1 1.1 1.1" color="#000" fill="#fff"/>
    ///   <path d="M80.22 13.93a1.1 1.1 0 0 1-1.1 1.1 1.1 1.1 0 0 1-1.08-1.1 1.1 1.1 0 0 1 1.1-1.08 1.1 1.1 0 0 1 1.08 1.08" color="#000" fill="none" stroke="#030303" stroke-linecap="round" stroke-opacity=".5"/>
    ///   <path d="M80.22 13.93a1.1 1.1 0 0 1-1.1 1.1 1.1 1.1 0 0 1-1.08-1.1 1.1 1.1 0 0 1 1.1-1.08 1.1 1.1 0 0 1 1.08 1.08" color="#000" fill="#fff"/>
    ///   <path d="M84.08 9.55a1.1 1.1 0 0 1-1.08 1.1 1.1 1.1 0 0 1-1.1-1.1 1.1 1.1 0 0 1 1.1-1.1 1.1 1.1 0 0 1 1.08 1.1" color="#000" fill="none" stroke="#030303" stroke-linecap="round" stroke-opacity=".5"/>
    ///   <path d="M84.08 9.55a1.1 1.1 0 0 1-1.08 1.1 1.1 1.1 0 0 1-1.1-1.1 1.1 1.1 0 0 1 1.1-1.1 1.1 1.1 0 0 1 1.08 1.1" color="#000" fill="#fff"/>
    ///   <path d="M89.1 6.66a1.1 1.1 0 0 1-1.08 1.1 1.1 1.1 0 0 1-1.08-1.1 1.1 1.1 0 0 1 1.08-1.08 1.1 1.1 0 0 1 1.1 1.08" color="#000" fill="none" stroke="#030303" stroke-linecap="round" stroke-opacity=".5"/>
    ///   <path d="M89.1 6.66a1.1 1.1 0 0 1-1.08 1.1 1.1 1.1 0 0 1-1.08-1.1 1.1 1.1 0 0 1 1.08-1.08 1.1 1.1 0 0 1 1.1 1.08" color="#000" fill="#fff"/>
    ///   <path d="M94.4 5a1.1 1.1 0 0 1-1.1 1.1A1.1 1.1 0 0 1 92.23 5a1.1 1.1 0 0 1 1.08-1.08A1.1 1.1 0 0 1 94.4 5" color="#000" fill="none" stroke="#030303" stroke-linecap="round" stroke-opacity=".5"/>
    ///   <path d="M94.4 5a1.1 1.1 0 0 1-1.1 1.1A1.1 1.1 0 0 1 92.23 5a1.1 1.1 0 0 1 1.08-1.08A1.1 1.1 0 0 1 94.4 5" color="#000" fill="#fff"/>
    ///   <path d="M76.44 25.13a1.1 1.1 0 0 1-1.1 1.1 1.1 1.1 0 0 1-1.08-1.1 1.1 1.1 0 0 1 1.1-1.1 1.1 1.1 0 0 1 1.08 1.1" color="#000" fill="none" stroke="#030303" stroke-linecap="round" stroke-opacity=".5"/>
    ///   <path d="M76.44 25.13a1.1 1.1 0 0 1-1.1 1.1 1.1 1.1 0 0 1-1.08-1.1 1.1 1.1 0 0 1 1.1-1.1 1.1 1.1 0 0 1 1.08 1.1" color="#000" fill="#fff"/>
    ///   <path d="M27.78 24.9a1.1 1.1 0 0 1-1.08 1.08 1.1 1.1 0 0 1-1.1-1.08 1.1 1.1 0 0 1 1.1-1.1 1.1 1.1 0 0 1 1.08 1.1" color="#000" fill="none" stroke="#030303" stroke-linecap="round" stroke-opacity=".5"/>
    ///   <path d="M27.78 24.9a1.1 1.1 0 0 1-1.08 1.08 1.1 1.1 0 0 1-1.1-1.08 1.1 1.1 0 0 1 1.1-1.1 1.1 1.1 0 0 1 1.08 1.1" color="#000" fill="#fff"/>
    ///   <path d="M45.4 5.14a1.1 1.1 0 0 1-1.08 1.1 1.1 1.1 0 0 1-1.1-1.1 1.1 1.1 0 0 1 1.1-1.08 1.1 1.1 0 0 1 1.1 1.08" color="#000" fill="none" stroke="#030303" stroke-linecap="round" stroke-opacity=".5"/>
    ///   <path d="M45.4 5.14a1.1 1.1 0 0 1-1.08 1.1 1.1 1.1 0 0 1-1.1-1.1 1.1 1.1 0 0 1 1.1-1.08 1.1 1.1 0 0 1 1.1 1.08" color="#000" fill="#fff"/>
    ///   <path d="M28.67 13.8a1.1 1.1 0 0 1-1.1 1.08 1.1 1.1 0 0 1-1.08-1.08 1.1 1.1 0 0 1 1.08-1.1 1.1 1.1 0 0 1 1.1 1.1" color="#000" fill="none" stroke="#030303" stroke-linecap="round" stroke-opacity=".5"/>
    ///   <path d="M28.67 13.8a1.1 1.1 0 0 1-1.1 1.08 1.1 1.1 0 0 1-1.08-1.08 1.1 1.1 0 0 1 1.08-1.1 1.1 1.1 0 0 1 1.1 1.1" color="#000" fill="#fff"/>
    ///   <path d="M35 7.32a1.1 1.1 0 0 1-1.1 1.1 1.1 1.1 0 0 1-1.08-1.1 1.1 1.1 0 0 1 1.1-1.1A1.1 1.1 0 0 1 35 7.33" color="#000" fill="none" stroke="#030303" stroke-linecap="round" stroke-opacity=".5"/>
    ///   <path d="M35 7.32a1.1 1.1 0 0 1-1.1 1.1 1.1 1.1 0 0 1-1.08-1.1 1.1 1.1 0 0 1 1.1-1.1A1.1 1.1 0 0 1 35 7.33" color="#000" fill="#fff"/>
    ///   <text style="line-height:6.61458302px" x="35.74" y="284.49" font-size="5.29" font-family="Sans" letter-spacing="0" word-spacing="0" fill="#b3b3b3" stroke-width=".26" transform="translate(19.595 -267)">
    ///     <tspan x="35.74" y="284.49" font-size="10.58">→</tspan>
    ///   </text>
    /// </svg>
    ///
    /// The tolerance threshold taken as input by the flattening algorithms corresponds
    /// to the maximum distance between the curve and its linear approximation.
    /// The smaller the tolerance is, the more precise the approximation and the more segments
    /// are generated. This value is typically chosen in function of the zoom level.
    ///
    /// <svg viewBox="0 0 47.5 13.2" height="100" width="350" xmlns="http://www.w3.org/2000/svg">
    ///   <path d="M-2.44 9.53c16.27-8.5 39.68-7.93 52.13 1.9" fill="none" stroke="#dde9af" stroke-width="4.6"/>
    ///   <path d="M-1.97 9.3C14.28 1.03 37.36 1.7 49.7 11.4" fill="none" stroke="#00d400" stroke-width=".57" stroke-linecap="round" stroke-dasharray="4.6, 2.291434"/>
    ///   <path d="M-1.94 10.46L6.2 6.08l28.32-1.4 15.17 6.74" fill="none" stroke="#000" stroke-width=".6"/>
    ///   <path d="M6.83 6.57a.9.9 0 0 1-1.25.15.9.9 0 0 1-.15-1.25.9.9 0 0 1 1.25-.15.9.9 0 0 1 .15 1.25" color="#000" stroke="#000" stroke-width=".57" stroke-linecap="round" stroke-opacity=".5"/>
    ///   <path d="M35.35 5.3a.9.9 0 0 1-1.25.15.9.9 0 0 1-.15-1.25.9.9 0 0 1 1.25-.15.9.9 0 0 1 .15 1.24" color="#000" stroke="#000" stroke-width=".6" stroke-opacity=".5"/>
    ///   <g fill="none" stroke="#ff7f2a" stroke-width=".26">
    ///     <path d="M20.4 3.8l.1 1.83M19.9 4.28l.48-.56.57.52M21.02 5.18l-.5.56-.6-.53" stroke-width=".2978872"/>
    ///   </g>
    /// </svg>
    ///
    /// The figure above shows a close up on a curve (the dotted line) and its linear
    /// approximation (the black segments). The tolerance threshold is represented by
    /// the light green area and the orange arrow.
    ///
    /// Credits for this figure go to [kurbo](https://docs.rs/kurbo/0.9.2/kurbo/struct.BezPath.html#method.flatten) or [lyon](https://docs.rs/lyon_geom/latest/lyon_geom/index.html), whoever had it first.
    pub fn flatten(&self, tolerance: f32, flattened_path: &mut Vec<FlattenedPathPoint>) {
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
                    flattened_path.push(FlattenedPathPoint::new(point, !start_new_contour));
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
                        flattened_path.push(FlattenedPathPoint::new(
                            curve_value_at_t,
                            !start_new_contour,
                        ));
                        start_new_contour = false;
                    }

                    // Connect to the end of the contour
                    flattened_path.push(FlattenedPathPoint::new(p2, !start_new_contour));

                    start_new_contour = false;
                },
            }
        }
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

    /// Compute a point along the curve.
    /// `t` should be between `0.0` and `1.0`.
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

#[derive(Clone, Copy, Debug)]
pub struct FlattenedPathPoint {
    pub coordinates: Vec2D,
    pub connected: bool,
}

impl FlattenedPathPoint {
    #[inline]
    pub fn new(coordinates: Vec2D, connected: bool) -> Self {
        Self {
            coordinates,
            connected,
        }
    }
}
