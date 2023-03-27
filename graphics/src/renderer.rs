use crate::Compositor;

/// Manages the rendering pipeline
#[derive(Clone, Copy, Debug)]
pub struct Renderer;

impl Renderer {
    pub fn render(compositor: &mut Compositor) {
        compositor.flatten_layers_if_necessary();
    }
}
