//! [Layer] management

use std::collections::{hash_map::Iter, HashMap};

use image::Texture;

use crate::Layer;

/// Manages all the different [Layers](Layer) that should be rendered.
///
/// Generally, there should never be a need to create more than one [Composition].
#[derive(Debug, Clone)]
pub struct Composition {
    dpi: (f32, f32),
    layers: HashMap<u16, Layer>,
}

impl Default for Composition {
    fn default() -> Self {
        Self {
            dpi: (1., 1.),
            layers: HashMap::default(),
        }
    }
}

impl Composition {
    /// Tries to retrieve the [Layer] at the given index in the composition.
    ///
    /// If there is no layer at the current index, a default layer is created and
    /// returned.
    pub fn get_or_insert_layer(&mut self, at_index: u16) -> &mut Layer {
        self.layers.entry(at_index).or_insert_with(|| {
            let mut new_layer = Layer::default();
            new_layer.scale(self.dpi.0, self.dpi.1);
            new_layer
        })
    }

    pub fn layers(&self) -> Iter<'_, u16, Layer> {
        self.layers.iter()
    }

    #[inline]
    pub fn set_dpi(&mut self, dpi: (f32, f32)) {
        self.dpi = dpi;
    }

    #[inline]
    pub fn clear(&mut self) {
        self.layers.clear();
    }

    pub fn render_to(&mut self, texture: &mut Texture) {
        // Draw all the layers, in order
        let mut keys: Vec<u16> = self.layers.keys().copied().collect();
        keys.sort();

        for key in keys {
            let layer = self
                .layers
                .get_mut(&key)
                .expect("Every key returned by layers.keys() should be valid");

            layer.render_to(texture);
        }
    }
}
