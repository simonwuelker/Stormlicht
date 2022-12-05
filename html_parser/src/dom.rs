use crate::tokenizer::TagData;
use std::cell::RefCell;
use std::rc::Rc;

pub type SharedDOMNode = Rc<RefCell<DOMNode>>;

// behaviour that is shared by all dom nodes goes here
#[derive(Debug, PartialEq)]
pub struct DOMNode {
    /// None for the root document node
    parent: Option<(SharedDOMNode, usize)>,
    pub node_type: DOMNodeType,
    children: Vec<SharedDOMNode>,
}

// node specific behaviour goes here
#[derive(Debug, PartialEq)]
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
    Base,
    BaseFont,
    BGSound,
    Link,
    Meta,
    Title,
    NoScript,
    Template,
    Body,
    Frameset,
    P,
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
    Foreign(String),
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

impl From<TagData> for DOMNode {
    fn from(from: TagData) -> Self {
        // Note that not all DOMNode's can be constructed from tagdata
        // For example, comments or DOCTYPEs cannot be created
        let domnode_type = match from.name.as_str() {
            "document" => DOMNodeType::Document,
            "html" => DOMNodeType::Html,
            "head" => DOMNodeType::Head,
            "base" => DOMNodeType::Base,
            "basefont" => DOMNodeType::BaseFont,
            "bgsound" => DOMNodeType::BGSound,
            "link" => DOMNodeType::Link,
            "meta" => DOMNodeType::Meta,
            "title" => DOMNodeType::Title,
            "noscript" => DOMNodeType::NoScript,
            "template" => DOMNodeType::Template,
            "body" => DOMNodeType::Body,
            "frameset" => DOMNodeType::Frameset,
            "p" => DOMNodeType::P,
            "h1" => DOMNodeType::H1,
            "h2" => DOMNodeType::H2,
            "h3" => DOMNodeType::H3,
            "h4" => DOMNodeType::H4,
            "h5" => DOMNodeType::H5,
            "h6" => DOMNodeType::H6,
            _ => DOMNodeType::Foreign(from.name),
        };
        DOMNode::new(domnode_type)
    }
}
