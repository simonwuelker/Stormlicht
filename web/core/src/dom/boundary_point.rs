use std::cmp::Ordering;

use crate::dom::{dom_objects, DOMPtr};

/// <https://dom.spec.whatwg.org/#boundary-points>
#[derive(Clone, Debug)]
pub struct BoundaryPoint {
    node: DOMPtr<dom_objects::Node>,
    offset: usize,
}

/// Describes the positions of two nodes within a tree relative to each other
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelativePosition {
    /// <https://dom.spec.whatwg.org/#concept-range-bp-before>
    Before,

    /// <https://dom.spec.whatwg.org/#concept-range-bp-equal>
    Equal,

    /// <https://dom.spec.whatwg.org/#concept-range-bp-after>
    After,
}

impl BoundaryPoint {
    #[inline]
    #[must_use]
    pub fn new(node: DOMPtr<dom_objects::Node>, offset: usize) -> Self {
        let length = dom_objects::Node::len(node.clone());
        assert!(offset < length);

        Self { node, offset }
    }

    /// <https://dom.spec.whatwg.org/#concept-range-bp-position>
    #[must_use]
    pub fn position_relative_to(&self, other: Self) -> RelativePosition {
        // 1. FIXME: Assert: nodeA and nodeB have the same root.

        // 2. If nodeA is nodeB, then return equal if offsetA is offsetB, before
        //    if offsetA is less than offsetB, and after if offsetA is greater than offsetB.
        if self.node.ptr_eq(&other.node) {
            return match self.offset.cmp(&other.offset) {
                Ordering::Equal => RelativePosition::Equal,
                Ordering::Less => RelativePosition::Before,
                Ordering::Greater => RelativePosition::After,
            };
        }

        // 3. FIXME: If nodeA is following nodeB, then if the position of (nodeB, offsetB) relative to (nodeA, offsetA) is before,
        //    return after, and if it is after, return before.

        // 4. If nodeA is an ancestor of nodeB:
        if let Some(child_index) =
            dom_objects::Node::find_subtree_containing(self.node.clone(), other.node)
        {
            // NOTE: find_subtree_containing does the next 3 steps for us
            // 1. Let child be nodeB.
            // 2. While child is not a child of nodeA, set child to its parent.
            // 3. If child’s index is less than offsetA, then return after.

            if child_index < self.offset {
                return RelativePosition::After;
            }
        }

        // 5. Return before.
        RelativePosition::Before
    }
}
