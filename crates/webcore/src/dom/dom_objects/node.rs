use dom_derive::inherit;
use std::fmt;

use super::{Comment, Document, Text};
use crate::{
    dom::{DomPtr, DomType, WeakDomPtr},
    TreeDebug, TreeFormatter,
};

/// <https://dom.spec.whatwg.org/#interface-node>
#[inherit]
pub struct Node {
    parent_node: Option<WeakDomPtr<Node>>,
    child_nodes: Vec<DomPtr<Node>>,
    owning_document: Option<WeakDomPtr<Document>>,
}

impl Node {
    /// <https://dom.spec.whatwg.org/#concept-node-length>
    #[must_use]
    pub fn len(node: DomPtr<Self>) -> usize {
        match node.underlying_type() {
            // 1. If node is a DocumentType or Attr node, then return 0.
            // FIXME: We don't have attr node's yet
            DomType::DocumentType => 0,

            // 2. If node is a CharacterData node, then return node’s data’s length.
            DomType::Comment => node.into_type::<Comment>().borrow().comment_data().len(),
            DomType::Text => node.into_type::<Text>().borrow().content().len(),

            // 3. Return the number of node’s children.
            _ => node.borrow().children().len(),
        }
    }

    /// <https://dom.spec.whatwg.org/#concept-node-empty>
    #[inline]
    #[must_use]
    pub fn is_empty(node: DomPtr<Self>) -> bool {
        Self::len(node) == 0
    }

    /// <https://dom.spec.whatwg.org/#concept-tree-ancestor>
    #[inline]
    #[must_use]
    pub fn is_ancestor_of(this: DomPtr<Self>, other: DomPtr<Self>) -> bool {
        Self::find_subtree_containing(this, other).is_some()
    }

    #[inline]
    #[must_use]
    pub fn find_subtree_containing(this: DomPtr<Self>, other: DomPtr<Self>) -> Option<usize> {
        if let Some(parent_node) = other.borrow().parent_node() {
            if DomPtr::ptr_eq(&this, &parent_node) {
                // Find the subtree within the list of our children
                let index = this
                    .borrow()
                    .children()
                    .iter()
                    .position(|child| DomPtr::ptr_eq(child, &parent_node))
                    .expect("Parent node does not contain child");

                Some(index)
            } else {
                Self::find_subtree_containing(this, parent_node)
            }
        } else {
            // "other" does not have a parent, so it can't be in any of our childrens subtrees
            None
        }
    }

    #[inline]
    #[must_use]
    pub fn children(&self) -> &[DomPtr<Self>] {
        &self.child_nodes
    }

    #[inline]
    #[must_use]
    pub fn parent_node(&self) -> Option<DomPtr<Node>> {
        self.parent_node.as_ref()?.upgrade()
    }

    // #[inline]
    // #[must_use]
    // pub fn first_child(&self) -> Option<DOMPtr<Node>> {
    //     self.child_nodes.first().cloned()
    // }

    pub fn last_child(&self) -> Option<DomPtr<Node>> {
        self.children().last().cloned()
    }

    pub fn append_child(parent: DomPtr<Node>, child: DomPtr<Node>) {
        child.borrow_mut().parent_node = Some(parent.downgrade());
        parent.borrow_mut().child_nodes.push(child);
    }

    pub fn owning_document(&self) -> Option<DomPtr<Document>> {
        self.owning_document.as_ref()?.upgrade()
    }

    pub fn set_owning_document(&mut self, document: WeakDomPtr<Document>) {
        self.owning_document = Some(document);
    }
}

impl fmt::Debug for DomPtr<Node> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut tree_formatter = TreeFormatter::new(f);
        self.tree_fmt(&mut tree_formatter)
    }
}
