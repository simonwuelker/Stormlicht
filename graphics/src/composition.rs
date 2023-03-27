//! [Layer] management

use std::collections::HashMap;

use crate::{AffineTransform, Color, FlattenedPathPoint, Path};

/// The maximum distance from the bezier curve to its flattened counterpart
const FLATTEN_TOLERANCE: f32 = 0.01;
/// Manages all the different [Layers](Layer) that should be rendered.
///
/// Generally, there should never be a need to create more than one [Compositor].
#[derive(Debug, Clone, Default)]
pub struct Compositor {
    layers: HashMap<usize, Layer>,
}

impl Compositor {
    /// Tries to retrieve the [Layer] at the given index in the composition.
    ///
    /// If there is no layer at the current index, a default layer is created and
    /// returned.
    pub fn get_or_insert_layer(&mut self, at_index: usize) -> &mut Layer {
        self.layers.entry(at_index).or_insert_with(Layer::default)
    }

    /// Update the internal list of flattened curves if the scale of the layer changed.
    /// If only translation/rotation changed, that is not necessary.
    pub fn flatten_layers_if_necessary(&mut self) {
        for layer in self.layers.values_mut() {
            layer.flatten_if_necessary();
        }
    }
}

/// A collection of [Path]'s which share common properties like a [Color] and [AffineTransform].
///
/// A [Layer] is constructed using [Compositor::get_or_insert_layer]
#[derive(Clone, Debug, Default)]
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

    /// Set a common transformation that should be applied to all elements in the layer
    #[inline]
    pub fn set_transform(&mut self, transform: AffineTransform) -> &mut Self {
        self.transform = transform;
        self
    }

    /// Add a contour to the layer
    #[inline]
    pub fn add_path(&mut self, path: Path) -> &mut Self {
        self.paths.push(path);
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
}
