use dom_derive::inherit;

use crate::{DOMPtr, WeakDOMPtr};

/// <https://dom.spec.whatwg.org/#interface-node>
#[inherit]
pub struct Node {
    parent_node: Option<WeakDOMPtr<Node>>,
}

impl Node {
    pub fn parent_node(&self) -> Option<DOMPtr<Node>> {
        self.parent_node.as_ref()?.upgrade()
    }
}
