use std::collections::HashMap;

#[derive(Debug)]
pub struct DomNode {
    node_type: NodeType,
    children: Vec<DomNode>,
}

#[derive(Debug)]
pub enum NodeType {
    Text(String),
    Complex(NodeData),
}

#[derive(Debug)]
pub struct NodeData {
    name: String,
    attributes: HashMap<String, String>,
}

impl DomNode {
    pub fn new(name: String) -> Self {
        Self {
            node_type: NodeType::Complex(NodeData {
                name: name,
                attributes: HashMap::new(),
            }),
            children: Vec::new(),
        }
    }

    pub fn text(text: String) -> Self {
        Self {
            node_type: NodeType::Text(text),
            children: Vec::new(),
        }
    }

    pub fn append(&mut self, child: DomNode) {
        self.children.push(child);
    }
}
