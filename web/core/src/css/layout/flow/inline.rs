use std::{fmt::Write, rc::Rc};

use font_metrics::FontMetrics;
use math::Vec2D;
use sl_std::{range::Range, slice::SubsliceOffset};

use crate::{
    css::{
        font_metrics,
        fragment_tree::{Fragment, TextFragment},
        layout::{CSSPixels, ContainingBlock},
        stylecomputer::ComputedStyle,
        LineBreakIterator,
    },
    dom::{dom_objects, DOMPtr},
    TreeDebug, TreeFormatter,
};

/// <https://drafts.csswg.org/css2/#inline-level-boxes>
#[derive(Clone, Debug)]
pub enum InlineLevelBox {
    InlineBox(InlineBox),
    TextRun(TextRun),
}

#[derive(Clone, Debug)]
pub struct TextRun {
    node: DOMPtr<dom_objects::Node>,
    text: String,
    selected_part: Option<Range<usize>>,
    style: Rc<ComputedStyle>,
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

impl TextRun {
    #[inline]
    #[must_use]
    pub fn new(
        node: DOMPtr<dom_objects::Node>,
        text: String,
        selected_part: Option<Range<usize>>,
        style: Rc<ComputedStyle>,
    ) -> Self {
        Self {
            node,
            text,
            selected_part,
            style,
        }
    }

    #[inline]
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    #[inline]
    #[must_use]
    pub fn style(&self) -> Rc<ComputedStyle> {
        self.style.clone()
    }
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
        containing_block: ContainingBlock,
    ) -> (Vec<Fragment>, CSSPixels) {
        let mut cursor = position;
        let mut fragments = vec![];
        let mut line_box_height = CSSPixels::ZERO;
        let mut has_seen_relevant_content = false;

        for inline_level_box in self.elements() {
            match inline_level_box {
                InlineLevelBox::TextRun(text_run) => {
                    // FIXME: Respect the elements actual font
                    let font_metrics = FontMetrics::default();

                    if line_box_height < font_metrics.size {
                        line_box_height = font_metrics.size;
                    }

                    // Collapse sequences of whitespace in the text
                    let mut previous_c_was_whitespace = true;
                    let mut text_without_whitespace_sequences = text_run.text().to_owned();
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

                    has_seen_relevant_content |=
                        collapsed_text.contains(|c: char| !c.is_whitespace());

                    // Fragment the text into individual lines
                    let available_width = containing_block.width();
                    let line_iterator = LineBreakIterator::new(
                        &collapsed_text,
                        font_metrics.clone(),
                        available_width,
                    );

                    for text_line in line_iterator {
                        let area = math::Rectangle {
                            top_left: cursor,
                            bottom_right: cursor
                                + Vec2D {
                                    x: text_line.width,
                                    y: font_metrics.size,
                                },
                        };

                        let line_range = collapsed_text
                            .subslice_range(text_line.text)
                            .expect("Broken line must be part of the original string");

                        let selected_part = text_run
                            .selected_part
                            .and_then(|range| range.intersection(line_range));

                        let fragment = Fragment::Text(TextFragment::new(
                            text_run.node.clone(),
                            line_range.start(),
                            text_line.text.to_string(),
                            area,
                            text_run.style().color(),
                            font_metrics.clone(),
                            selected_part,
                        ));
                        fragments.push(fragment);

                        cursor.x = position.x;
                        cursor.y += font_metrics.size;
                    }
                },
                InlineLevelBox::InlineBox(_inline_box) => {
                    // has_seen_relevant_content = true;
                    todo!("fragment inline boxes")
                },
            }
        }

        if has_seen_relevant_content {
            (fragments, cursor.y - position.y)
        } else {
            (vec![], CSSPixels::ZERO)
        }
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
            Self::TextRun(text_run) => {
                formatter.indent()?;
                formatter.write_text(text_run.text())?;
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
