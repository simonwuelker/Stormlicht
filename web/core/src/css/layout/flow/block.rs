use std::rc::Rc;

use crate::{
    css::{stylecomputer::ComputedStyle, StyleComputer},
    dom::{dom_objects, DOMPtr},
};

use super::{BoxTreeBuilder, InlineFormattingContext, InlineLevelBox};

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
///
/// <https://drafts.csswg.org/css2/#block-container-box>
#[derive(Clone)]
pub enum BlockContainer {
    BlockLevelBoxes(Vec<BlockLevelBox>),
    InlineFormattingContext(InlineFormattingContext),
}

impl Default for BlockContainer {
    fn default() -> Self {
        Self::InlineFormattingContext(vec![].into())
    }
}

impl BlockFormattingContext {
    pub fn root(document: DOMPtr<dom_objects::Node>, style_computer: StyleComputer<'_>) -> Self {
        let document_style =
            Rc::new(style_computer.get_computed_style(document.clone().into_type()));

        let contents =
            BoxTreeBuilder::build(document.clone(), style_computer, document_style.clone());
        let root = BlockLevelBox {
            style: document_style,
            contents,
            node: Some(document),
        };

        vec![root].into()
    }
}

impl From<Vec<BlockLevelBox>> for BlockFormattingContext {
    fn from(contents: Vec<BlockLevelBox>) -> Self {
        Self { contents }
    }
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

    pub fn from_element(
        element: DOMPtr<dom_objects::Element>,
        style: Rc<ComputedStyle>,
        style_computer: StyleComputer<'_>,
    ) -> Self {
        debug_assert!(style.display().is_block());

        let node: DOMPtr<dom_objects::Node> = element.into_type();

        let mut block_box = Self {
            node: Some(node.clone()),
            style,
            contents: BlockContainer::default(),
        };

        for child in node.borrow().children() {
            if let Some(element) = child.try_into_type() {
                // Compute the style for the child
                let style = Rc::new(style_computer.get_computed_style(element.clone()));

                // If the child has display:none then it does not generate any boxes
                if style.display().is_none() {
                    continue;
                }

                // FIXME: display: contents

                if style.display().is_inline() {
                    // let inline_level_box = InlineLevelBox::from_element(element, style.clone());
                    // block_box.push_inline_box(inline_level_box)
                } else {
                    let block_level_box =
                        BlockLevelBox::from_element(element, style.clone(), style_computer);
                    block_box.push_block_box(block_level_box);
                }
            }
        }

        block_box
    }

    pub fn create_anonymous_box(contents: BlockContainer, style: Rc<ComputedStyle>) -> Self {
        Self {
            style,
            node: None,
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
    /// Appends a block-level child to the box
    ///
    /// The boxes contents are adjusted as necessary.
    #[inline]
    pub fn push_block_box(&mut self, block_level_box: BlockLevelBox) {
        self.contents
            .push_block_box(block_level_box, self.style.clone())
    }

    /// Appends a inline-level child to the box
    ///
    /// The boxes contents are adjusted as necessary.
    #[inline]
    pub fn push_inline_box(&mut self, inline_level_box: InlineLevelBox) {
        self.contents
            .push_inline_box(inline_level_box, self.style.clone())
    }
}

impl BlockContainer {
    /// Appends a block-level child box to the container
    ///
    /// If the block previously only contained inline elements,
    /// all of those are wrapped in anonymous block boxes.
    ///
    /// You'll likely want to use [BlockLevelBox::push_block_box] instead.
    fn push_block_box(
        &mut self,
        block_level_box: BlockLevelBox,
        containing_box_style: Rc<ComputedStyle>,
    ) {
        match self {
            Self::BlockLevelBoxes(block_level_boxes) => {
                block_level_boxes.push(block_level_box);
            },
            Self::InlineFormattingContext(inline_formatting_context) => {
                // Once *one* child is block-level, all of them need to be block-level too.
                let mut block_level_boxes: Vec<_> = inline_formatting_context
                    .elements()
                    .iter()
                    .map(|inline_box| {
                        BlockLevelBox::create_anonymous_wrapper_around(
                            inline_box.clone(),
                            containing_box_style.clone(),
                        )
                    })
                    .collect();

                // Add the current element to the list of block boxes
                block_level_boxes.push(block_level_box);

                *self = Self::BlockLevelBoxes(block_level_boxes);
            },
        }
    }

    /// Appends a inline-level child box to the container
    ///
    /// You'll likely want to use [BlockLevelBox::push_inline_box]
    fn push_inline_box(
        &mut self,
        inline_level_box: InlineLevelBox,
        containing_box_style: Rc<ComputedStyle>,
    ) {
        match self {
            Self::BlockLevelBoxes(block_level_boxes) => {
                // Wrap an anonymous block-level box around the inline box
                let block_level_box = BlockLevelBox::create_anonymous_wrapper_around(
                    inline_level_box,
                    containing_box_style.clone(),
                );
                block_level_boxes.push(block_level_box);
            },
            Self::InlineFormattingContext(inline_formatting_context) => {
                inline_formatting_context.push(inline_level_box)
            },
        }
    }
}
