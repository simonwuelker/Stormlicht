use crate::{consts, math::Vec2D, Compositor};

#[derive(Clone, Copy, Debug)]
pub(crate) struct LineSegment {
    /// x expressed with respect to a parameter t
    pub x_t: LinearRelation,
    /// y expressed with respect to a parameter t
    pub y_t: LinearRelation,
    /// t expressed with respect to x
    pub t_x: LinearRelation,
    /// t expressed with respect to y
    pub t_y: LinearRelation,
    /// The exact number of pixels touched by this line
    pub number_of_pixels: u32,
    pub layer_index: u16,
}

/// Describes a linear relation between two values.
///
/// `y = at * t + b`
#[derive(Clone, Copy, Debug)]
pub(crate) struct LinearRelation {
    pub slope: f32,
    /// The value of `y` at `t = 0`
    pub offset: f32,
}

impl LinearRelation {
    #[inline]
    pub(crate) fn evaluate_at(&self, t: f32) -> f32 {
        self.slope.mul_add(t, self.offset)
    }

    #[inline]
    pub(crate) fn solve_for(&self, y: f32) -> f32 {
        (y - self.offset) / self.slope
    }
}

pub(crate) fn compute_line_segments(
    compositor: &mut Compositor,
    viewport_width: usize,
    viewport_height: usize,
) -> Vec<LineSegment> {
    let width = viewport_width as f32;
    let height = viewport_height as f32;

    let mut line_segments = vec![];
    for (&layer_index, layer) in compositor.layers() {
        for line in layer.flattened_path().array_windows::<2>() {
            // Skip lines between two contours
            if !line[1].connected {
                continue;
            }

            // Apply the transformation assigned to that layer
            let transform = layer.get_transform();
            let p0 = transform.apply_to(line[0].coordinates);
            let p1 = transform.apply_to(line[1].coordinates);

            // Skip lines outside the viewport
            if line_is_outside_viewport(p0, p1, width, height) {
                continue;
            }

            // Horizontal lines are not relevant for rasterization
            if p0.y == p1.y {
                continue;
            }

            // Here, we express the line with two linear equations with respect to an extra parameter t:
            // x(t) = t / dx + start_x
            // y(t) = t / dy + start_y
            // This allows us to evaluate any point along the line based on t
            // start_x and start_y are also pixel-aligned.

            let delta_x = p1.x - p0.x;
            let delta_y = p1.y - p0.y;

            // Compute start_x, start_y such that x(t_x) and y(t_y) are on a pixel boundary
            // while changing the line as little as possible
            let start_x = (p0.x.floor() - p0.x).max(p0.x.ceil() - p0.x) / delta_x;
            let start_y = (p0.y.floor() - p0.y).max(p0.y.ceil() - p0.y) / delta_y;

            let number_of_pixels =
                number_of_integers_between(p0.x, p1.x) + number_of_integers_between(p0.y, p1.y) + 1;

            line_segments.push(LineSegment {
                x_t: LinearRelation {
                    slope: delta_x,
                    offset: p0.x * consts::PIXEL_SIZE as f32,
                },
                y_t: LinearRelation {
                    slope: delta_y,
                    offset: p0.y * consts::PIXEL_SIZE as f32,
                },
                t_x: LinearRelation {
                    slope: delta_x.recip(),
                    offset: start_x,
                },
                t_y: LinearRelation {
                    slope: delta_y.recip(),
                    offset: start_y,
                },
                number_of_pixels,
                layer_index,
            })
        }
    }
    line_segments
}

/// NOTE: This function isn't perfect. The following example shows a line that would not be flagged:
/// ```text, ignore
///             x (p0)
///               \
///                \
/// ╔══════════════╗\
/// ║              ║ \
/// ║  <viewport>  ║  \
/// ║              ║   x (p1)
/// ╚══════════════╝
/// ```
/// This is fine, since it's only used for optimization and should prioritize speed over correctness.
fn line_is_outside_viewport(p0: Vec2D, p1: Vec2D, width: f32, height: f32) -> bool {
    // Note that we can't ignore lines to the left of the viewport since those
    // might start a contour that reaches into the viewport

    p0.y.is_sign_negative() && p1.y.is_sign_negative()  // Line is above the viewport
        || width < p0.x && width < p1.x  // Line is to the right of the viewport
        || height < p0.y && height < p1.y // Line is below the viewport
}

#[inline]
fn number_of_integers_between(a: f32, b: f32) -> u32 {
    let min = a.min(b);
    let max = a.max(b);
    ((max.ceil() - min.floor()) as u32).max(1) - 1
}

#[cfg(test)]
mod tests {
    #[test]
    fn number_of_integers_between() {
        assert_eq!(super::number_of_integers_between(-0.5, 0.99), 1);
        assert_eq!(super::number_of_integers_between(0.5, 0.99), 0);
        assert_eq!(super::number_of_integers_between(3.4, 1.1), 2);
        assert_eq!(super::number_of_integers_between(1.0, 1.0), 0);
    }
}
