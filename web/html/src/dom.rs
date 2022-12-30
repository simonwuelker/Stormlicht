use std::rc::{Rc, Weak};
use crate::dom_ptr::*;
use std::any::TypeId;

macro_rules! id {
	($tt: tt) => (TypeId::of::<S>())
}

#[derive(Debug)]
pub enum NodeType {
	Element,
	Attribute,
	Text,
	CDATASection,
	EntityReference,
	Entity,
	ProcessingInstruction,
	Comment,
	Document,
	DocumentType,
	DocumentFragment,
	Notation,
}

/// https://dom.spec.whatwg.org/#node
#[repr(C)]
#[derive(Debug)]
pub struct Node {
	pub node_type: NodeType,
}

/// https://dom.spec.whatwg.org/#interface-document
#[repr(C)]
#[derive(Debug)]
pub struct Document {
	pub node: Node,
	pub x: (),
}

/// https://dom.spec.whatwg.org/#interface-element
#[repr(C)]
#[derive(Debug)]
pub struct Element {
	pub node: Node,
	pub x: (),
}

impl SuperClassOf<Document> for Node {}
impl SuperClassOf<Element> for Node {}

impl Document {
	pub fn new() -> DOMPtr<Self> {
		DOMPtr::new(Self {
			node: Node {
				node_type: NodeType::Document,
			},
			x: (),
		})
	}
}

// TODO autogenerate this, much safer that way
pub fn inheritance_match<S: 'static>(id: TypeId) -> bool {
	if id!(S) == id!(Node) {
		id == id!(Node) ||id == id!(Document) || id == id!(Element)
	} else if id!(S) == id!(Document) {
		id == id!(Document)
	} else if id!(S) == id!(Element) {
		id == id!(Element)
	}
	else {
		false
	}
}