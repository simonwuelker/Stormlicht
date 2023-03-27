use crate::Vec2D;

/// A 2-dimensional transformation.
///
/// Each [AffineTransform] is a `3x3` matrix that transforms a 2 dimensional vector `x`, `y`.
/// See [Wikipedia](https://en.wikipedia.org/wiki/Affine_transformation) for more information.
pub struct AffineTransform([[f32; 3]; 2]);

impl AffineTransform {
    #[inline]
    pub const fn identity() -> Self {
        Self([[1., 0., 0.], [0., 1., 0.]])
    }

    /// Create transformation that shifts every point by a fixed offset
    #[inline]
    pub const fn translate(translate_by: Vec2D) -> Self {
        Self([[1., 0., translate_by.x], [0., 1., translate_by.y]])
    }

    /// Create a transformation that scales points by fixed values along the X and Y axis
    #[inline]
    pub const fn scale(x_scale: f32, y_scale: f32) -> Self {
        Self([[x_scale, 0., 0.], [0., y_scale, 0.]])
    }

    /// Create a transformation that rotates points counterclockwise around the origin by a fixed
    /// amount
    #[inline]
    pub fn rotate(angle: f32) -> Self {
        Self([
            [angle.cos(), -angle.sin(), 0.],
            [angle.sin(), angle.cos(), 0.],
        ])
    }

    /// Apply this transform to a provided vector
    #[inline]
    pub fn apply_to(self, point: Vec2D) -> Vec2D {
        Vec2D {
            x: point
                .x
                .mul_add(self.0[0][0], point.y.mul_add(self.0[0][1], self.0[0][2])),
            y: point
                .x
                .mul_add(self.0[1][0], point.y.mul_add(self.0[1][1], self.0[1][2])),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AffineTransform;
    use crate::Vec2D;

    #[test]
    fn test_identity() {
        let transform = AffineTransform::identity();
        let point = Vec2D::new(2., 3.);
        assert_eq!(transform.apply_to(point), point);
    }

    #[test]
    fn test_translate() {
        let transform = AffineTransform::translate(Vec2D::new(1., 2.));
        let point = Vec2D::new(4., -3.);
        assert_eq!(transform.apply_to(point), Vec2D::new(5., -1.));
    }

    #[test]
    fn test_scale() {
        let transform = AffineTransform::scale(2., -1.);
        let point = Vec2D::new(2., 2.);
        assert_eq!(transform.apply_to(point), Vec2D::new(4., -2.));
    }
}
