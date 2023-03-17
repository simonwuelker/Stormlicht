use dom_derive::inherit;

use crate::{DOMPtr, WeakDOMPtr};

use super::Document;

/// <https://dom.spec.whatwg.org/#interface-node>
#[inherit]
pub struct Node {
    parent_node: Option<WeakDOMPtr<Node>>,
    child_nodes: Vec<DOMPtr<Node>>,
    owning_document: Option<WeakDOMPtr<Document>>,
}

impl Node {
    pub fn parent_node(&self) -> Option<DOMPtr<Node>> {
        self.parent_node.as_ref()?.upgrade()
    }

    pub fn first_child(&self) -> Option<DOMPtr<Node>> {
        self.child_nodes.first().cloned()
    }

    pub fn last_child(&self) -> Option<DOMPtr<Node>> {
        self.child_nodes.first().cloned()
    }

    pub fn append_child(parent: DOMPtr<Node>, child: DOMPtr<Node>) {
        child.borrow_mut().parent_node = Some(parent.downgrade());
        parent.borrow_mut().child_nodes.push(child);
    }

    pub fn owning_document(&self) -> Option<DOMPtr<Document>> {
        self.owning_document.as_ref()?.upgrade()
    }

    pub fn set_owning_document(&mut self, document: WeakDOMPtr<Document>) {
        self.owning_document = Some(document);
    }
}
