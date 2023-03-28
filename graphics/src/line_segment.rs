use crate::{Compositor, Vec2D};

#[derive(Clone, Copy, Debug)]
pub(crate) struct LineSegment {
    pub p0: Vec2D,
    pub delta_x: f32,
    pub delta_y: f32,
    /// x expressed with respect to a parameter t
    pub x_t: LinearRelation,
    /// y expressed with respect to a parameter t
    pub y_t: LinearRelation,
}

/// Describes a linear relation between two values.
#[derive(Clone, Copy, Debug)]
pub(crate) struct LinearRelation {
    pub slope: f32,
    pub y_offset: f32,
}

pub(crate) fn compute_line_segments(
    compositor: &mut Compositor,
    viewport_width: usize,
    viewport_height: usize,
) -> Vec<LineSegment> {
    let width = viewport_width as f32;
    let height = viewport_height as f32;

    let mut line_segments = vec![];
    for layer in compositor.layers() {
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

            // Compute start_x, start_y such that x(t_x) and y(t_y) are pixel aligned
            // while changing the line as little as possible
            let start_x = (p0.x.round() - p0.x) / delta_x;
            let start_y = (p0.y.round() - p0.y) / delta_y;

            line_segments.push(LineSegment {
                p0,
                delta_x,
                delta_y,
                x_t: LinearRelation {
                    slope: delta_x.recip(),
                    y_offset: start_x,
                },
                y_t: LinearRelation {
                    slope: delta_y.recip(),
                    y_offset: start_y,
                },
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
