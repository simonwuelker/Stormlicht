use std::cell::RefCell;
use std::rc::Rc;

pub type SharedDOMNode = Rc<RefCell<DOMNode>>;

// behaviour that is shared by all dom nodes goes here
pub struct DOMNode {
    /// None for the root document node
    parent: Option<(SharedDOMNode, usize)>,
    node_type: DOMNodeType,
    children: Vec<SharedDOMNode>,
}

// node specific behaviour goes here
pub enum DOMNodeType {
    Document,
    Comment {
        data: String,
    },
    DocumentType {
        name: String,
        public_ident: String,
        system_ident: String,
    },
    Html,
    Head,
    Text(String),
}

impl DOMNode {
    pub fn new(node_type: DOMNodeType) -> Self {
        Self {
            parent: None,
            node_type: node_type,
            children: vec![],
        }
    }

    pub fn add_child(parent: SharedDOMNode, child: SharedDOMNode) {
        let child_index = parent.borrow().children.len();
        child.borrow_mut().parent = Some((parent.clone(), child_index));
        parent.borrow_mut().children.push(child);
    }

    pub fn to_shared(self) -> SharedDOMNode {
        Rc::new(RefCell::new(self))
    }

    pub fn previous_sibling(&self) -> Option<SharedDOMNode> {
        if let Some((parent_node, index)) = &self.parent {
            if *index != 0 {
                Some(parent_node.borrow().children[index - 1].clone())
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn add_text(&self, text: String) {
        // The root document node cannot hold text nodes
        if let Some((parent, _)) = &self.parent {
            if let Some(shared_sibling) = self.previous_sibling() {
                let mut sibling = shared_sibling.borrow_mut();
                if let DOMNodeType::Text(ref mut existing_text) = &mut sibling.node_type {
                    existing_text.push_str(&text);
                    return;
                }
            }

            // Create a new TextNode
            let text_node = DOMNode::new(DOMNodeType::Text(text)).to_shared();
            Self::add_child(parent.clone(), text_node);
        }
    }
}
