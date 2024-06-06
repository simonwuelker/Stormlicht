//! Contains fragmented Boxes
//!
//! The layout engine produces a fragment tree, which consists
//! of (regular) boxes and line boxes (from a fragmented text run)

mod fragment;

pub use fragment::{BoxFragment, Fragment, TextFragment, TextureFragment};

use super::{
    display_list::Painter,
    layout::{Pixels, Size},
};

#[derive(Clone, Copy, Debug)]
struct DisplayState {
    has_seen_background_on_html_element: bool,
    viewport: Size<Pixels>,
    offset: math::Vec2D<Pixels>,
}

#[derive(Clone, Debug, Default)]
pub struct FragmentTree {
    root_fragments: Vec<Fragment>,
}

impl FragmentTree {
    #[must_use]
    pub fn new(root_fragments: Vec<Fragment>) -> Self {
        Self { root_fragments }
    }

    #[must_use]
    pub fn hit_test(&self, position: math::Vec2D<Pixels>) -> Option<&'_ Fragment> {
        self.root_fragments
            .iter()
            .rev() // Traverse in reverse paint order
            .filter_map(|fragment| fragment.hit_test(position))
            .next()
    }

    pub fn fill_display_list(&self, painter: &mut Painter, viewport: Size<Pixels>) {
        let mut state = DisplayState {
            has_seen_background_on_html_element: false,
            viewport,
            offset: math::Vec2D::new(Pixels::ZERO, Pixels::ZERO),
        };

        for fragment in &self.root_fragments {
            fragment.fill_display_list(painter, &mut state);
        }
    }
}
