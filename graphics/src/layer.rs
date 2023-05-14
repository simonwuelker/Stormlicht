pub use crate::{
    math::{AffineTransform, Angle},
    Color, Path,
};
use crate::{
    math::{Rectangle, Vec2D},
    Buffer, FlattenedPathPoint, Rasterizer,
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
    pub fn with_source(&mut self, source: Source) -> &mut Self {
        self.source = source;
        self
    }

    /// Set the outline of the layer
    #[inline]
    pub fn with_outline(&mut self, path: Path) -> &mut Self {
        self.outline = path;
        self
    }

    /// Rotate the layer by a fixed angle
    ///
    /// This operation does not cause the Bézier curves to be re-flattened
    #[inline]
    pub fn rotate(&mut self, angle: Angle) -> &mut Self {
        self.transform = self.transform.chain(AffineTransform::rotate(angle));
        self
    }

    /// Move the layer by a fixed amount
    ///
    /// This operation does not cause the Bézier curves to be re-flattened
    #[inline]
    pub fn translate(&mut self, translate_by: Vec2D) -> &mut Self {
        self.transform = self
            .transform
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

        self.transform = self
            .transform
            .chain(AffineTransform::scale(x_scale, y_scale));
        self.needs_flattening = true;
        self
    }

    fn flatten_if_necessary(&mut self) {
        const FLATTEN_TOLERANCE: f32 = 0.01;
        if self.needs_flattening {
            self.flattened_outline.clear();
            self.outline
                .flatten(FLATTEN_TOLERANCE, &mut self.flattened_outline)
        }
    }

    fn apply_transform(&mut self) -> Option<Rectangle> {
        let mut extents: Option<Rectangle> = None;
        for p in &mut self.flattened_outline {
            p.coordinates = self.transform.apply_to(p.coordinates);

            match extents {
                Some(ref mut rectangle) => {
                    rectangle.top_left.x = rectangle.top_left.x.min(p.coordinates.x);
                    rectangle.top_left.y = rectangle.top_left.y.min(p.coordinates.y);
                    rectangle.bottom_right.x = rectangle.bottom_right.x.max(p.coordinates.x);
                    rectangle.bottom_right.y = rectangle.bottom_right.x.max(p.coordinates.y);
                },
                None => {
                    extents = Some(Rectangle {
                        top_left: p.coordinates,
                        bottom_right: p.coordinates,
                    })
                },
            }
        }
        extents
    }

    pub(crate) fn render_to(&mut self, buffer: &mut Buffer) {
        self.flatten_if_necessary();

        if let Some(outline_extent) = self.apply_transform() {
            // Compute a mask for the layer.
            // This mask determines which pixels in the buffer should be
            // colored and which should not be.
            let outline_offset = outline_extent.top_left;
            let outline_extent = outline_extent.round_to_grid();
            let mut rasterizer = Rasterizer::new(outline_extent, outline_offset);
            rasterizer.fill(&self.flattened_outline);
            let mask = rasterizer.into_mask();

            // Compose the mask onto the buffer
            buffer.compose(mask, self.source, outline_extent.top_left);
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
