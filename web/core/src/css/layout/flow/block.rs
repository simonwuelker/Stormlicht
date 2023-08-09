use std::{fmt, fmt::Write, rc::Rc};

use crate::{
    css::{stylecomputer::ComputedStyle, StyleComputer},
    dom::{dom_objects, DOMPtr},
    TreeDebug, TreeFormatter,
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
    #[must_use]
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

    #[inline]
    #[must_use]
    pub fn is_anonymous(&self) -> bool {
        self.node.is_none()
    }

    #[inline]
    #[must_use]
    pub fn style(&self) -> Rc<ComputedStyle> {
        self.style.clone()
    }

    #[must_use]
    pub fn create_anonymous_box(contents: BlockContainer, style: Rc<ComputedStyle>) -> Self {
        Self {
            style,
            node: None,
            contents,
        }
    }

    #[must_use]
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

impl fmt::Debug for BlockFormattingContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut tree_formatter = TreeFormatter::new(f);
        self.tree_fmt(&mut tree_formatter)
    }
}

impl TreeDebug for BlockFormattingContext {
    fn tree_fmt(&self, formatter: &mut TreeFormatter<'_, '_>) -> std::fmt::Result {
        formatter.indent()?;
        writeln!(formatter, "Block Formatting Context")?;
        formatter.increase_indent();
        for child in &self.contents {
            child.tree_fmt(formatter)?;
        }
        formatter.decrease_indent();
        Ok(())
    }
}

impl TreeDebug for BlockLevelBox {
    fn tree_fmt(&self, formatter: &mut TreeFormatter<'_, '_>) -> std::fmt::Result {
        formatter.indent()?;
        write!(formatter, "Block Box")?;
        if let Some(node) = &self.node {
            writeln!(formatter, " ({:?})", node.underlying_type())?;
        } else {
            writeln!(formatter, " (anonymous)")?;
        }

        formatter.increase_indent();
        match &self.contents {
            BlockContainer::BlockLevelBoxes(block_level_boxes) => {
                for block_box in block_level_boxes {
                    block_box.tree_fmt(formatter)?;
                }
            },
            BlockContainer::InlineFormattingContext(inline_formatting_context) => {
                inline_formatting_context.tree_fmt(formatter)?;
            },
        }
        formatter.decrease_indent();
        Ok(())
    }
}
