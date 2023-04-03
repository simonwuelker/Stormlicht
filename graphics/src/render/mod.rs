//! The rasterization pipeline

pub mod line_segment;
pub mod pixel_segment;

use crate::{
    render::line_segment::compute_line_segments, render::pixel_segment::rasterize_line_segments,
    Compositor,
};

/// Manages the rendering pipeline
#[derive(Clone, Copy, Debug)]
pub struct Renderer;

impl Renderer {
    pub fn render(compositor: &mut Compositor, width: usize, height: usize) {
        // If any paths were added, or the scale transform changed, we need to reflatten
        // some paths
        compositor.flatten_layers_if_necessary();

        // Convert the flattened path into pixel-aligned line segments
        // This also removes any lines that are not relevant for the render
        let line_segments = compute_line_segments(compositor, width, height);

        // Convert the line segments into tiles of 16x16 pixels
        let _pixel_segments = rasterize_line_segments(line_segments);
    }
}
