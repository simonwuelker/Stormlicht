//! Contains fragmented Boxes
//!
//! The layout engine produces a fragment tree, which consists
//! of (regular) boxes and line boxes (from a fragmented text run)

mod fragment;

pub use fragment::{BoxFragment, Fragment, TextFragment};

use super::display_list::Painter;

#[derive(Clone, Debug)]
pub struct FragmentTree {
    root_fragments: Vec<Fragment>,
}

impl FragmentTree {
    pub fn new(root_fragments: Vec<Fragment>) -> Self {
        Self { root_fragments }
    }

    pub fn fill_display_list(&self, painter: &mut Painter) {
        for fragment in &self.root_fragments {
            fragment.fill_display_list(painter);
        }
    }
}
