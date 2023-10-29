//! Contains fragmented Boxes
//!
//! The layout engine produces a fragment tree, which consists
//! of (regular) boxes and line boxes (from a fragmented text run)

mod fragment;

pub use fragment::{BoxFragment, Fragment, TextFragment};

use crate::dom;

use super::{display_list::Painter, layout::CSSPixels};

#[derive(Clone, Debug, Default)]
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

    pub fn hit_test(&self, point: math::Vec2D<CSSPixels>) -> Option<dom::BoundaryPoint> {
        for fragment in &self.root_fragments {
            if fragment
                .content_area_including_overflow()
                .contains_point(point)
            {
                if let Some(hit) = fragment.hit_test(point) {
                    return Some(hit);
                }
            }
        }
        None
    }
}
