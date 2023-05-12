//! [Layer] management

use std::collections::{hash_map::Iter, HashMap};

use crate::Layer;

/// Manages all the different [Layers](Layer) that should be rendered.
///
/// Generally, there should never be a need to create more than one [Composition].
#[derive(Debug, Clone, Default)]
pub struct Composition {
    layers: HashMap<u16, Layer>,
}

impl Composition {
    /// Tries to retrieve the [Layer] at the given index in the composition.
    ///
    /// If there is no layer at the current index, a default layer is created and
    /// returned.
    pub fn get_or_insert_layer(&mut self, at_index: u16) -> &mut Layer {
        self.layers.entry(at_index).or_insert_with(Layer::default)
    }

    pub fn layers(&self) -> Iter<'_, u16, Layer> {
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
