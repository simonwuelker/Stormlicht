//! [Layer] management

use std::collections::{hash_map::Iter, HashMap};

use crate::{
    math::{AffineTransform, Angle, Vec2D},
    Color, FlattenedPathPoint, Path,
};

/// The maximum distance from the bezier curve to its flattened counterpart
const FLATTEN_TOLERANCE: f32 = 0.01;
/// Manages all the different [Layers](Layer) that should be rendered.
///
/// Generally, there should never be a need to create more than one [Compositor].
#[derive(Debug, Clone, Default)]
pub struct Compositor {
    layers: HashMap<u32, Layer>,
}

impl Compositor {
    /// Tries to retrieve the [Layer] at the given index in the composition.
    ///
    /// If there is no layer at the current index, a default layer is created and
    /// returned.
    pub fn get_or_insert_layer(&mut self, at_index: u32) -> &mut Layer {
        self.layers.entry(at_index).or_insert_with(Layer::default)
    }

    pub fn layers(&self) -> Iter<'_, u32, Layer> {
        self.layers.iter()
    }

    /// Update the internal list of flattened curves if the scale of the layer changed.
    /// If only translation/rotation changed, that is not necessary.
    pub fn flatten_layers_if_necessary(&mut self) {
        for layer in self.layers.values_mut() {
            if layer.is_enabled {
                layer.flatten_if_necessary();
            }
        }
    }
}

/// A collection of [Path]'s which share common properties like a [Color] and [AffineTransform].
///
/// A [Layer] is constructed using [Compositor::get_or_insert_layer]
#[derive(Clone, Debug)]
pub struct Layer {
    /// Controls whether or not a [Layer]'s contents should be rendered to the screen
    is_enabled: bool,

    /// The graphical elements within the layer
    paths: Vec<Path>,

    /// The color that the renderer should use for drawing the elements
    color: Color,

    /// A common transformation applied to all elements in the layer
    transform: AffineTransform,
    needs_flattening: bool,
    flattened_path: Vec<FlattenedPathPoint>,
}

impl Default for Layer {
    fn default() -> Self {
        Self {
            is_enabled: true,
            paths: vec![],
            color: Color::default(),
            transform: AffineTransform::identity(),
            needs_flattening: false,
            flattened_path: vec![],
        }
    }
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

    /// Set the color of the elements within the [Layer]
    #[inline]
    pub fn set_color(&mut self, color: Color) -> &mut Self {
        self.color = color;
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

    /// Add a contour to the layer
    #[inline]
    pub fn add_path(&mut self, path: Path) -> &mut Self {
        self.paths.push(path);
        self.needs_flattening = true;
        self
    }

    fn flatten_if_necessary(&mut self) {
        if self.needs_flattening {
            self.flattened_path.clear();
            for path in &mut self.paths {
                path.flatten(FLATTEN_TOLERANCE, &mut self.flattened_path)
            }
        }
    }

    /// Get the layers transform
    ///
    /// Modifying the transform directly on the layer causes bugs because `self.needs_reflattening`
    /// wouldn't be updated. That's even other modules within this crate don't ever get mutable access
    /// to the transfor. The transform should **only** be updated by the compositor.
    #[inline]
    pub(crate) fn get_transform(&self) -> AffineTransform {
        self.transform
    }

    #[inline]
    pub(crate) fn flattened_path(&self) -> &[FlattenedPathPoint] {
        &self.flattened_path
    }
}
