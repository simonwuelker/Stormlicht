//! Contains fragmented Boxes
//!
//! The layout engine produces a fragment tree, which consists
//! of (regular) boxes and line boxes (from a fragmented text run)

mod fragment;

pub use fragment::{BoxFragment, Fragment, LineBoxFragment};

use super::display_list::Painter;

#[derive(Clone, Debug)]
pub struct FragmentTree {
    pub root_fragments: Vec<Fragment>,
}

impl FragmentTree {
    pub fn fill_display_list(&self, painter: &mut Painter) {
        for fragment in &self.root_fragments {
            fragment.fill_display_list(painter);
        }
    }
}
