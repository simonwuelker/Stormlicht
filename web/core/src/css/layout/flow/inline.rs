use std::{fmt::Write, rc::Rc};

use crate::{css::stylecomputer::ComputedStyle, TreeDebug, TreeFormatter};

/// <https://drafts.csswg.org/css2/#inline-level-boxes>
#[derive(Clone, Debug)]
pub enum InlineLevelBox {
    InlineBox(InlineBox),
    TextRun(String),
}

/// <https://drafts.csswg.org/css2/#inline-box>
#[derive(Clone, Debug)]
pub struct InlineBox {
    style: Rc<ComputedStyle>,
    contents: Vec<InlineLevelBox>,
}

/// <https://drafts.csswg.org/css2/#inline-formatting>
#[derive(Clone, Debug, Default)]
pub struct InlineFormattingContext {
    elements: Vec<InlineLevelBox>,
}

impl InlineFormattingContext {
    #[inline]
    pub fn elements(&self) -> &[InlineLevelBox] {
        &self.elements
    }

    #[inline]
    pub fn push(&mut self, inline_level_box: InlineLevelBox) {
        self.elements.push(inline_level_box)
    }

    /// Return true if there are no elements in the [InlineFormattingContext]
    ///
    /// Note that a valid [InlineFormattingContext] should never be empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }
}

impl From<Vec<InlineLevelBox>> for InlineFormattingContext {
    fn from(elements: Vec<InlineLevelBox>) -> Self {
        Self { elements }
    }
}

impl InlineBox {
    #[inline]
    pub fn new(style: Rc<ComputedStyle>) -> Self {
        Self {
            style,
            contents: Vec::new(),
        }
    }

    #[inline]
    pub fn push(&mut self, element: InlineLevelBox) {
        self.contents.push(element);
    }

    /// Create a inline box with the same style but no children
    ///
    /// This is necessary when an [InlineBox] needs to be split due to
    /// a [BlockLevelBox](super::BlockLevelBox) inside it.
    #[inline]
    pub fn split_off(&self) -> Self {
        Self {
            style: self.style.clone(),
            contents: vec![],
        }
    }
}

impl TreeDebug for InlineFormattingContext {
    fn tree_fmt(&self, formatter: &mut TreeFormatter<'_, '_>) -> std::fmt::Result {
        writeln!(formatter, "Inline Formatting Context")?;
        formatter.increase_indent();
        for child in &self.elements {
            child.tree_fmt(formatter)?;
        }
        Ok(())
    }
}

impl TreeDebug for InlineLevelBox {
    fn tree_fmt(&self, formatter: &mut TreeFormatter<'_, '_>) -> std::fmt::Result {
        match self {
            Self::TextRun(text) => writeln!(formatter, "{:?}", text)?,
            Self::InlineBox(inline_box) => {
                writeln!(formatter, "Inline Box");
                formatter.increase_indent();
                for child in &inline_box.contents {
                    child.tree_fmt(formatter)?;
                }
                formatter.decrease_indent();
            }
        }
        Ok(())
    }
}
