//! Contains [BoxTreeBuilder]
//!
//! Thanks to servo, the basic builder algorithm is the same as theirs
//! <https://github.com/servo/servo/blob/main/components/layout_2020/flow/construct.rs>

use std::rc::Rc;

use crate::{
    css::{
        layout::flow::{
            BlockContainer, BlockLevelBox, InlineBox, InlineFormattingContext, InlineLevelBox,
        },
        stylecomputer::ComputedStyle,
        StyleComputer,
    },
    dom::{dom_objects, DOMPtr},
};

use super::TextRun;

#[derive(Clone)]
pub struct BoxTreeBuilder<'a> {
    style_computer: StyleComputer<'a>,
    style: Rc<ComputedStyle>,
    block_level_boxes: Vec<BlockLevelBox>,
    current_inline_formatting_context: InlineFormattingContext,
    inline_stack: Vec<InlineBox>,
}

impl<'a> BoxTreeBuilder<'a> {
    pub fn build(
        node: DOMPtr<dom_objects::Node>,
        style_computer: StyleComputer<'a>,
        style: Rc<ComputedStyle>,
    ) -> BlockContainer {
        let mut builder = Self {
            style_computer,
            style: style.clone(),
            block_level_boxes: Vec::new(),
            current_inline_formatting_context: InlineFormattingContext::default(),
            inline_stack: Vec::new(),
        };

        builder.traverse_subtree(node, style);

        if !builder.current_inline_formatting_context.is_empty() {
            if builder.block_level_boxes.is_empty() {
                return BlockContainer::InlineFormattingContext(
                    builder.current_inline_formatting_context,
                );
            }
            builder.end_inline_formatting_context();
        }

        BlockContainer::BlockLevelBoxes(builder.block_level_boxes)
    }

    fn traverse_subtree(
        &mut self,
        node: DOMPtr<dom_objects::Node>,
        parent_style: Rc<ComputedStyle>,
    ) {
        for child in node.borrow().children() {
            if let Some(element) = child.try_into_type::<dom_objects::Element>() {
                let computed_style = Rc::new(self.style_computer.get_computed_style(element));

                if computed_style.display().is_none() {
                    continue;
                }

                if computed_style.display().is_inline() {
                    self.push_inline_box(child.clone(), computed_style);
                } else {
                    self.push_block_box(child.clone(), computed_style);
                }
            } else if let Some(text) = child.try_into_type::<dom_objects::Text>() {
                // Content that would later be collapsed away according to the white-space property
                // does not generate inline boxes
                let text = text.borrow();
                if text.content().contains(|c: char| !c.is_whitespace()) {
                    let text_run = TextRun::new(text.content().to_owned(), parent_style.clone());
                    self.push_text(text_run);
                }
            }
        }
    }

    /// Wrap the current inline formatting context in a block level box and push
    fn end_inline_formatting_context(&mut self) {
        debug_assert!(!self.current_inline_formatting_context.is_empty());

        let formatting_context = std::mem::take(&mut self.current_inline_formatting_context);
        self.block_level_boxes
            .push(BlockLevelBox::create_anonymous_box(
                BlockContainer::InlineFormattingContext(formatting_context),
                self.style.clone(),
            ));
    }

    fn push_text(&mut self, text_run: TextRun) {
        let text_box = InlineLevelBox::TextRun(text_run);

        if let Some(top_box) = self.inline_stack.last_mut() {
            top_box.push(text_box);
        } else {
            // inline box stack is empty
            self.current_inline_formatting_context.push(text_box);
        }
    }

    fn push_inline_box(&mut self, node: DOMPtr<dom_objects::Node>, style: Rc<ComputedStyle>) {
        self.inline_stack
            .push(InlineBox::new(node.clone(), style.clone()));

        // Traverse all children, they will be appended to the inline box we just created
        self.traverse_subtree(node, style);

        // Pop the inline box from the stack and append it to its parents list of children
        // unless the stack of open inline boxes is empty, in which case this was a top level box
        // and we append it to the ongoing inline formatting context instead
        let populated_inline_box = InlineLevelBox::InlineBox(
            self.inline_stack
                .pop()
                .expect("stack of open inline boxes should not be empty"),
        );

        if let Some(top_box) = self.inline_stack.last_mut() {
            top_box.push(populated_inline_box);
        } else {
            // inline box stack is empty
            self.current_inline_formatting_context
                .push(populated_inline_box);
        }
    }

    fn push_block_box(&mut self, node: DOMPtr<dom_objects::Node>, style: Rc<ComputedStyle>) {
        // Split all currently open inline boxes around the block box
        if !self.inline_stack.is_empty() {
            // Split each inline box - these will end up on the "right side" of the block box
            let mut fragments = self.inline_stack.iter().map(InlineBox::split_off).collect();

            std::mem::swap(&mut self.inline_stack, &mut fragments);

            // Push the fragmented tree on the "left side" of the block box to the current
            // inline formatting context
            let left_side_box = fragments
                .into_iter()
                .reduce(|child_tree, mut parent| {
                    parent.push(InlineLevelBox::InlineBox(child_tree));
                    parent
                })
                .expect("inline box stack cannot be empty");
            self.current_inline_formatting_context
                .push(InlineLevelBox::InlineBox(left_side_box));
        }

        // End the current inline formatting context, as we've found a block box
        // that interrupts the sequence of inline boxes
        if !self.current_inline_formatting_context.is_empty() {
            self.end_inline_formatting_context();
        }

        // Push the actual box
        let contents = Self::build(node.clone(), self.style_computer, style.clone());
        self.block_level_boxes
            .push(BlockLevelBox::new(style, Some(node), contents));
    }
}
