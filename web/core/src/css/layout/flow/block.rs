use std::rc::Rc;

use crate::{
    css::{stylecomputer::ComputedStyle, StyleComputer},
    dom::{dom_objects, DOMPtr},
};

use super::{InlineFormattingContext, InlineLevelBox};

/// <https://drafts.csswg.org/css2/#block-formatting>
#[derive(Clone)]
pub struct BlockFormattingContext {
    contents: Vec<BlockLevelBox>,
}

/// A Box that participates in a [BlockFormattingContext]
/// <https://drafts.csswg.org/css2/#block-level-boxes>
#[derive(Clone)]
pub struct BlockLevelBox {
    style: Rc<ComputedStyle>,

    /// The DOM element that produced this box.
    /// Some boxes might not correspond to a DOM node,
    /// for example anonymous block boxes
    node: Option<DOMPtr<dom_objects::Node>>,

    /// Boxes contained by this box
    contents: BlockContainer,
}

/// Elements contained in a [BlockLevelBox]
/// <https://drafts.csswg.org/css2/#block-container-box>
#[derive(Clone)]
pub enum BlockContainer {
    BlockLevelBoxes(Vec<BlockLevelBox>),
    InlineFormattingContext(InlineFormattingContext),
}

impl BlockLevelBox {
    pub fn new(
        style: Rc<ComputedStyle>,
        node: Option<DOMPtr<dom_objects::Node>>,
        contents: BlockContainer,
    ) -> Self {
        Self {
            style,
            node,
            contents,
        }
    }
}

impl From<Vec<BlockLevelBox>> for BlockFormattingContext {
    fn from(contents: Vec<BlockLevelBox>) -> Self {
        Self { contents }
    }
}

#[derive(Clone)]
enum BlockContainerBuilder {
    NoBlockLevelBoxesSoFar(Vec<InlineLevelBox>),
    FoundBlockLevelBox(Vec<BlockLevelBox>),
}

impl BlockLevelBox {
    pub fn from_element(
        element: DOMPtr<dom_objects::Element>,
        style: Rc<ComputedStyle>,
        style_computer: StyleComputer<'_>,
    ) -> Self {
        debug_assert!(style.display().is_block());

        let node: DOMPtr<dom_objects::Node> = element.into_type();
        let contents =
            BlockContainer::from_children(node.borrow().children().iter().cloned(), style_computer);
        Self {
            style,
            node: Some(node),
            contents,
        }
    }

    pub fn create_anonymous_wrapper_around(
        inline_box: InlineLevelBox,
        style: Rc<ComputedStyle>,
    ) -> Self {
        Self {
            style: style,
            node: None,
            contents: BlockContainer::InlineFormattingContext(vec![inline_box].into()),
        }
    }
}

impl Default for BlockContainerBuilder {
    fn default() -> Self {
        Self::NoBlockLevelBoxesSoFar(vec![])
    }
}

impl BlockContainer {
    fn from_children<I: Iterator<Item = DOMPtr<dom_objects::Node>>>(
        children: I,
        style_computer: StyleComputer<'_>,
    ) -> Self {
        // There are two possible results from constructing a block container:
        // Either all children were inline, in which case we put them all into an inline formatting
        // context - Or at least one of them was block-level, in which case we wrap any inline boxes in
        // anonymous block boxes
        let mut state = BlockContainerBuilder::default();

        for child in children {
            if let Some(element) = child.try_into_type() {
                // Compute the style for the child
                let style = Rc::new(style_computer.get_computed_style(element.clone()));

                // If the child has display:none then it does not generate any boxes
                if style.display().is_none() {
                    continue;
                }

                // FIXME: display: contents

                match state {
                    BlockContainerBuilder::FoundBlockLevelBox(ref mut block_level_boxes) => {
                        if style.display().is_inline() {
                            // Wrap an anonymous block-level box around the inline box
                            let inline_box = InlineLevelBox::from_element(element, style.clone());
                            let block_level_box =
                                BlockLevelBox::create_anonymous_wrapper_around(inline_box, style);
                            block_level_boxes.push(block_level_box);
                        } else {
                            let block_level_box =
                                BlockLevelBox::from_element(element, style.clone(), style_computer);
                            block_level_boxes.push(block_level_box);
                        }
                    },
                    BlockContainerBuilder::NoBlockLevelBoxesSoFar(ref mut inline_level_boxes) => {
                        if style.display().is_block() {
                            // Once *one* child is block-level, all of them need to be block-level too.
                            let mut block_boxes: Vec<_> = inline_level_boxes
                                .iter()
                                .map(|inline_box| {
                                    BlockLevelBox::create_anonymous_wrapper_around(
                                        inline_box.clone(),
                                        style.clone(),
                                    )
                                })
                                .collect();

                            // Add the current element to the list of block boxes
                            block_boxes.push(BlockLevelBox::from_element(
                                element,
                                style,
                                style_computer,
                            ));
                            state = BlockContainerBuilder::FoundBlockLevelBox(block_boxes);
                        } else {
                            let inline_box = InlineLevelBox::from_element(element, style.clone());
                            inline_level_boxes.push(inline_box);
                        }
                    },
                }
            }
        }

        state.into()
    }
}

impl From<BlockContainerBuilder> for BlockContainer {
    fn from(value: BlockContainerBuilder) -> Self {
        match value {
            BlockContainerBuilder::FoundBlockLevelBox(block_level_boxes) => {
                Self::BlockLevelBoxes(block_level_boxes)
            },
            BlockContainerBuilder::NoBlockLevelBoxesSoFar(inline_level_boxes) => {
                Self::InlineFormattingContext(inline_level_boxes.into())
            },
        }
    }
}
