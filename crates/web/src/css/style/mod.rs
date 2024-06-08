use super::layout::{Pixels, Size};

pub mod computed;
pub mod specified;

trait ToComputedStyle {
    type Computed;

    fn to_computed_style(&self, context: StyleContext) -> Self::Computed;
}

/// Contains all the data that a specified property could need to computed its computed value
pub struct StyleContext {
    pub font_size: Pixels,

    pub root_font_size: Pixels,

    /// The size of the viewport
    ///
    /// Viewport-relative units like `vw` depend on this
    pub viewport: Size<Pixels>,
}
