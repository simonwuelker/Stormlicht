use crate::{
    css::{stylecomputer::ComputedStyle, StyleComputer},
    dom::{dom_objects, DOMPtr},
};

use super::{block::BlockFormattingContext, BlockContainer, BlockLevelBox};

#[derive(Clone, Debug)]
pub struct BoxTree {
    root: BlockFormattingContext,
}

impl BoxTree {
    pub fn new(root: DOMPtr<dom_objects::Node>, computed_style: ComputedStyle) -> Self {
        let root_children = BlockContainer::BlockLevelBoxes(vec![]);
        let root_box = BlockLevelBox::new(root_children);
        Self {
            root: BlockFormattingContext::new(vec![root_box]),
        }
    }
}

#[derive(Clone, Debug)]
struct Visitor<'a> {
    style_computer: &'a StyleComputer<'a>,
    current_block: BlockContainer,
}

// Block container:
// Block level -> append
// Inline

impl<'a> Visitor<'a> {
    pub fn visit_node(&self, element: DOMPtr<dom_objects::Element>) {
        let computed_style = self.style_computer.get_computed_style(element.clone());
        if computed_style.display().is_none() {
            // Neither this element nor its descendents generate any boxes
            return;
        }
        if computed_style.display().is_contents() {
            for child in element.borrow().children() {
                if let Some(element) = child.try_into_type() {
                    self.visit_node(element);
                }
            }
        }
    }
}
