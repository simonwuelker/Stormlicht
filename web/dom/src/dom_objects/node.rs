use dom_derive::inherit;

use crate::DOMPtr;

/// <https://dom.spec.whatwg.org/#interface-node>
#[inherit]
pub struct Node {
    parent_node: Option<DOMPtr<Node>>,
}

impl Node {
    pub fn parent_node(&self) -> Option<&DOMPtr<Node>> {
        self.parent_node.as_ref()
    }
}
