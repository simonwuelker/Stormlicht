use crate::{math::Vec2D, FlattenedPathPoint};
pub use crate::{
    math::{AffineTransform, Angle},
    Color, Path,
};

#[derive(Clone, Copy, Debug)]
pub enum Source {
    /// One single color
    Solid(Color), // TODO: add more sources, like images and gradients
}

impl Default for Source {
    fn default() -> Self {
        Self::Solid(Color::default())
    }
}

#[derive(Clone, Debug)]
pub struct Layer {
    pub outline: Path,
    pub source: Source,

    /// A common transformation applied to all elements in the layer
    transform: AffineTransform,

    /// Controls whether or not a [Layer]'s contents should be rendered to the screen
    pub is_enabled: bool,
    needs_flattening: bool,
    flattened_outline: Vec<FlattenedPathPoint>,
}

impl Layer {
    /// Show the layer
    #[inline]
    pub fn enable(&mut self) -> &mut Self {
        self.is_enabled = true;
        self
    }

    /// Hide the layer
    #[inline]
    pub fn disable(&mut self) -> &mut Self {
        self.is_enabled = false;
        self
    }

    /// Set the color source of the elements within the [Layer]
    #[inline]
    pub fn set_source(&mut self, source: Source) -> &mut Self {
        self.source = source;
        self
    }

    /// Rotate the layer by a fixed angle
    ///
    /// This operation does not cause the Bézier curves to be re-flattened
    #[inline]
    pub fn rotate(&mut self, angle: Angle) -> &mut Self {
        self.transform.chain(AffineTransform::rotate(angle));
        self
    }

    /// Move the layer by a fixed amount
    ///
    /// This operation does not cause the Bézier curves to be re-flattened
    #[inline]
    pub fn translate(&mut self, translate_by: Vec2D) -> &mut Self {
        self.transform
            .chain(AffineTransform::translate(translate_by));
        self
    }

    /// Scale the layer by a fixed amount along both axis
    ///
    /// This operation causes the Bézier curves to be re-flattened
    #[inline]
    pub fn scale(&mut self, x_scale: f32, y_scale: f32) -> &mut Self {
        if x_scale == 1. && y_scale == 1. {
            return self;
        }

        self.transform
            .chain(AffineTransform::scale(x_scale, y_scale));
        self.needs_flattening = true;
        self
    }

    pub fn flatten_if_necessary(&mut self) {
        const FLATTEN_TOLERANCE: f32 = 0.01;
        if self.needs_flattening {
            self.outline
                .flatten(FLATTEN_TOLERANCE, &mut self.flattened_outline)
        }
    }
}

impl Default for Layer {
    fn default() -> Self {
        Self {
            outline: Path::empty(),
            source: Source::default(),
            transform: AffineTransform::identity(),
            is_enabled: true,
            needs_flattening: true,
            flattened_outline: vec![],
        }
    }
}
