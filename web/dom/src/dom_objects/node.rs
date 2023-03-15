use crate::DomPtr;

/// https://dom.spec.whatwg.org/#interface-node
pub struct Node {
    parent_node: Option<DomPtr<Node>>,
}

impl Node {
    pub fn parent_node(&self) -> Option<&DomPtr<Node>> {
        self.parent_node.as_ref()
    }
}
