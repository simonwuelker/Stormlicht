use std::{fmt::Write, rc::Rc};

use font_metrics::FontMetrics;
use math::Vec2D;

use crate::{
    css::{
        font_metrics,
        fragment_tree::{Fragment, TextFragment},
        layout::{CSSPixels, ContainingBlock},
        stylecomputer::ComputedStyle,
        values::color::Color,
    },
    dom::{dom_objects, DOMPtr},
    TreeDebug, TreeFormatter,
};

/// <https://drafts.csswg.org/css2/#inline-level-boxes>
#[derive(Clone, Debug)]
pub enum InlineLevelBox {
    InlineBox(InlineBox),
    TextRun(String),
}

/// <https://drafts.csswg.org/css2/#inline-box>
#[derive(Clone, Debug)]
pub struct InlineBox {
    node: DOMPtr<dom_objects::Node>,
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

    pub fn fragment(
        &self,
        position: Vec2D<CSSPixels>,
        _containing_block: ContainingBlock,
    ) -> (Vec<Fragment>, CSSPixels) {
        let mut cursor = position;
        let mut fragments = vec![];
        let mut line_box_height = CSSPixels::ZERO;
        for inline_level_box in self.elements() {
            match inline_level_box {
                InlineLevelBox::TextRun(text) => {
                    // FIXME: Respect the elements actual font
                    let font_metrics = FontMetrics::default();
                    let width = font_metrics.width_of(text);

                    if line_box_height < font_metrics.size {
                        line_box_height = font_metrics.size;
                    }

                    // Collapse sequences of whitespace in the text
                    let mut previous_c_was_whitespace = true;
                    let mut text_without_whitespace_sequences = text.clone();
                    text_without_whitespace_sequences.retain(|c| {
                        let is_whitespace = c.is_whitespace();
                        let retain = !is_whitespace || !previous_c_was_whitespace;
                        previous_c_was_whitespace = is_whitespace;
                        retain
                    });

                    // The previous collapse algorithm also removed leading whitespace, now we just need to remove
                    // trailing whitespace
                    let collapsed_text = text_without_whitespace_sequences
                        .as_str()
                        .trim_end()
                        .to_string();

                    let fragment = Fragment::Text(TextFragment::new(
                        collapsed_text,
                        cursor,
                        Color::BLACK,
                        font_metrics,
                    ));
                    fragments.push(fragment);

                    cursor.x += width;
                },
                InlineLevelBox::InlineBox(_inline_box) => {
                    todo!("fragment inline boxes")
                },
            }
        }
        (fragments, line_box_height)
    }
}

impl From<Vec<InlineLevelBox>> for InlineFormattingContext {
    fn from(elements: Vec<InlineLevelBox>) -> Self {
        Self { elements }
    }
}

impl InlineBox {
    #[inline]
    pub fn new(node: DOMPtr<dom_objects::Node>, style: Rc<ComputedStyle>) -> Self {
        Self {
            node,
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
            node: self.node.clone(),
            style: self.style.clone(),
            contents: vec![],
        }
    }
}

impl TreeDebug for InlineFormattingContext {
    fn tree_fmt(&self, formatter: &mut TreeFormatter<'_, '_>) -> std::fmt::Result {
        formatter.indent()?;
        writeln!(formatter, "Inline Formatting Context")?;
        formatter.increase_indent();
        for child in &self.elements {
            child.tree_fmt(formatter)?;
        }
        formatter.decrease_indent();
        Ok(())
    }
}

impl TreeDebug for InlineLevelBox {
    fn tree_fmt(&self, formatter: &mut TreeFormatter<'_, '_>) -> std::fmt::Result {
        match self {
            Self::TextRun(text) => {
                formatter.indent()?;
                formatter.write_text(text)?;
                writeln!(formatter)?;
            },
            Self::InlineBox(inline_box) => {
                formatter.indent()?;
                writeln!(
                    formatter,
                    "Inline Box ({:?})",
                    inline_box.node.underlying_type()
                )?;
                formatter.increase_indent();
                for child in &inline_box.contents {
                    child.tree_fmt(formatter)?;
                    writeln!(formatter)?;
                }
                formatter.decrease_indent();
            },
        }
        Ok(())
    }
}
