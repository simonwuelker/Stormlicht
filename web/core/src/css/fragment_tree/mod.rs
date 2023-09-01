//! Contains fragmented Boxes
//!
//! The layout engine produces a fragment tree, which consists
//! of (regular) boxes and line boxes (from a fragmented text run)

mod fragment;

pub use fragment::{BoxFragment, Fragment, LineBoxFragment};

#[derive(Clone, Debug)]
pub struct FragmentTree {
    pub root_fragments: Vec<Fragment>,
}
