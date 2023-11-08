use std::{fmt::Write, mem};

use font_metrics::FontMetrics;
use math::{Rectangle, Vec2D};

use crate::{
    css::{
        font_metrics::{self, DEFAULT_FONT_SIZE},
        fragment_tree::{BoxFragment, Fragment, TextFragment},
        layout::{CSSPixels, ContainingBlock, Sides, Size},
        values::{font_size, length},
        ComputedStyle, LineBreakIterator,
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
    text: String,
    style: ComputedStyle,
}

/// <https://drafts.csswg.org/css2/#inline-box>
#[derive(Clone, Debug)]
pub struct InlineBox {
    node: DOMPtr<dom_objects::Node>,
    style: ComputedStyle,
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
    pub fn new(text: String, style: ComputedStyle) -> Self {
        Self { text, style }
    }

    #[inline]
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    #[inline]
    #[must_use]
    pub fn style(&self) -> &ComputedStyle {
        &self.style
    }

    fn layout_into_line_items(&self, state: &mut InlineFormattingContextState) {
        // FIXME: use the inherited font size here, not the default one
        let ctx = font_size::ResolutionContext {
            inherited_font_size: DEFAULT_FONT_SIZE,
            length_context: length::ResolutionContext {
                viewport: state.viewport,
            },
        };

        let font_size = self.style().font_size().to_pixels(ctx);

        let font_metrics = FontMetrics::new(font_size);
        let height = font_metrics.size;

        // Collapse sequences of whitespace in the text and remove newlines as defined in
        // https://drafts.csswg.org/css2/#white-space-model (3)

        let mut previous_c_was_whitespace = false;
        let mut text_without_whitespace_sequences = self.text().to_owned();
        text_without_whitespace_sequences.retain(|c| {
            let is_whitespace = c.is_whitespace();
            let retain = !is_whitespace || !previous_c_was_whitespace;
            previous_c_was_whitespace = is_whitespace;
            retain && c != '\n'
        });

        let remaining_text = &text_without_whitespace_sequences;
        let mut lines = LineBreakIterator::new(
            remaining_text,
            font_metrics.clone(),
            state.remaining_width_for_line_box(),
        );

        while let Some(text_line) = lines.next() {
            // https://drafts.csswg.org/css2/#white-space-model
            //
            // 1. If a space (U+0020) at the beginning of a line has white-space set to normal,
            //    nowrap, or pre-line, it is removed.
            let visual_text = if state.at_beginning_of_line {
                text_line.text.trim_start().to_owned()
            } else {
                text_line.text.to_owned()
            };

            let line_item = LineItem::TextRun(TextRunItem {
                metrics: font_metrics.clone(),
                text: visual_text,
                width: text_line.width,
                style: self.style().get_inherited(),
            });
            state.push_line_item(line_item, text_line.width, height);

            if !lines.is_done() {
                state.finish_current_line();
                lines.adjust_available_width(state.remaining_width_for_line_box());
            }
        }
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

    pub fn layout(
        &self,
        position: Vec2D<CSSPixels>,
        containing_block: ContainingBlock,
        viewport: Size<CSSPixels>,
    ) -> (Vec<Fragment>, CSSPixels) {
        let mut state = InlineFormattingContextState::new(position, containing_block, viewport);

        state.traverse(self.elements());

        state.finish_current_line();

        if state.has_seen_relevant_content {
            (state.finished_fragments, state.y_cursor - position.y)
        } else {
            (vec![], CSSPixels::ZERO)
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct LineBoxUnderConstruction {
    height: CSSPixels,
    width: CSSPixels,
}

/// State of an IFC for the current nesting level
#[derive(Clone, Debug, Default)]
struct NestingLevelState {
    line_items: Vec<LineItem>,
}

#[derive(Clone, Debug)]
struct InlineBoxContainerState<'box_tree> {
    inline_box: &'box_tree InlineBox,
    nesting_level_state: NestingLevelState,
}

#[derive(Clone, Debug)]
struct InlineFormattingContextState<'box_tree> {
    /// Information about the line box currently being constructed
    line_box_under_construction: LineBoxUnderConstruction,

    root_nesting_level_state: NestingLevelState,

    /// A stack of inline boxes that were opened within this IFC
    inline_box_stack: Vec<InlineBoxContainerState<'box_tree>>,

    containing_block: ContainingBlock,
    finished_fragments: Vec<Fragment>,
    has_seen_relevant_content: bool,

    /// The top left corner of the first line box
    position: Vec2D<CSSPixels>,
    y_cursor: CSSPixels,

    /// `true` if the current line is empty
    ///
    /// This is necessary because whitespace at the beginning of a line is removed
    at_beginning_of_line: bool,

    viewport: Size<CSSPixels>,
}

#[derive(Clone, Debug)]
enum LineItem {
    TextRun(TextRunItem),
    InlineBox(InlineBoxItem),
}

/// A piece of text that takes up at most one line
#[derive(Clone, Debug)]
struct TextRunItem {
    metrics: FontMetrics,
    text: String,
    width: CSSPixels,
    style: ComputedStyle,
}

#[derive(Clone, Debug)]
struct InlineBoxItem {
    style: ComputedStyle,
    children: Vec<LineItem>,
}

/// State used during conversion from [LineItems](LineItem) to [Fragments](Fragment)
#[derive(Clone, Copy, Debug)]
struct LineItemLayoutState {
    top_left: Vec2D<CSSPixels>,
    width: CSSPixels,
    height: CSSPixels,
}

impl LineItemLayoutState {
    fn new(top_left: Vec2D<CSSPixels>) -> Self {
        Self {
            top_left,
            width: CSSPixels::ZERO,
            height: CSSPixels::ZERO,
        }
    }
}

impl InlineBoxItem {
    fn layout(self, state: &mut LineItemLayoutState) -> Option<BoxFragment> {
        // Create a nested layout state that will contain the children
        let start = Vec2D {
            x: state.top_left.x + state.width,
            y: state.top_left.y,
        };
        let mut nested_state = LineItemLayoutState::new(start);
        let child_fragments = nested_state.layout(self.children);

        if child_fragments.is_empty() {
            return None;
        }

        state.width += nested_state.width;
        if nested_state.height > state.height {
            state.height = nested_state.height;
        }

        let bottom_right = start
            + Vec2D {
                x: nested_state.width,
                y: nested_state.height,
            };
        let content_area = Rectangle {
            top_left: start,
            bottom_right,
        };

        // FIXME: respect margin for inline boxes
        let margin = Sides::all(CSSPixels::ZERO);

        let box_fragment = BoxFragment::new(
            None,
            self.style,
            margin,
            content_area,
            content_area,
            child_fragments,
        );

        Some(box_fragment)
    }
}

impl<'box_tree> InlineFormattingContextState<'box_tree> {
    fn new(
        position: Vec2D<CSSPixels>,
        containing_block: ContainingBlock,
        viewport: Size<CSSPixels>,
    ) -> Self {
        Self {
            line_box_under_construction: LineBoxUnderConstruction::default(),
            root_nesting_level_state: NestingLevelState::default(),
            inline_box_stack: Vec::new(),
            containing_block,
            finished_fragments: Vec::new(),
            has_seen_relevant_content: false,
            position: position,
            y_cursor: position.y,
            at_beginning_of_line: true,
            viewport,
        }
    }

    fn push_line_item(&mut self, line_item: LineItem, width: CSSPixels, height: CSSPixels) {
        self.line_box_under_construction.width += width;
        self.has_seen_relevant_content = true;
        self.at_beginning_of_line = false;

        if self.line_box_under_construction.height < height {
            self.line_box_under_construction.height = height;
        }

        self.current_insertion_point().line_items.push(line_item);
    }

    fn remaining_width_for_line_box(&self) -> CSSPixels {
        self.containing_block.width() - self.line_box_under_construction.width
    }

    fn traverse<I: IntoIterator<Item = &'box_tree InlineLevelBox>>(&mut self, iterator: I) {
        for element in iterator {
            match element {
                InlineLevelBox::InlineBox(inline_box) => {
                    self.start_inline_box(inline_box);
                    self.traverse(&inline_box.contents);
                    self.finish_inline_box();
                },
                InlineLevelBox::TextRun(text_run) => {
                    text_run.layout_into_line_items(self);
                },
            }
        }
    }

    fn finish_current_line(&mut self) {
        // Create LineItems for all boxes on the stack
        let mut inline_box_stack = self.inline_box_stack.iter_mut().rev();

        if let Some(top_box) = inline_box_stack.next() {
            let mut line_item = top_box.layout_into_line_item().into();

            for inline_box in inline_box_stack {
                inline_box.nesting_level_state.line_items.push(line_item);
                line_item = inline_box.layout_into_line_item().into();
            }

            self.root_nesting_level_state.line_items.push(line_item);
        }

        let items_on_this_line = mem::take(&mut self.root_nesting_level_state.line_items);

        let mut layout_state = LineItemLayoutState::new(Vec2D {
            x: self.position.x,
            y: self.y_cursor,
        });
        self.finished_fragments
            .extend(layout_state.layout(items_on_this_line));

        self.y_cursor += self.line_box_under_construction.height;

        // Prepare for a new line
        self.line_box_under_construction = LineBoxUnderConstruction {
            width: CSSPixels::ZERO,
            height: CSSPixels::ZERO,
        };

        self.at_beginning_of_line = true;
    }

    fn current_insertion_point(&mut self) -> &mut NestingLevelState {
        match self.inline_box_stack.last_mut() {
            Some(last_box) => &mut last_box.nesting_level_state,
            None => &mut self.root_nesting_level_state,
        }
    }

    fn start_inline_box(&mut self, inline_box: &'box_tree InlineBox) {
        self.inline_box_stack
            .push(InlineBoxContainerState::new(inline_box));
    }

    fn finish_inline_box(&mut self) {
        // Take the box that was closed
        let mut finished_box = match self.inline_box_stack.pop() {
            Some(inline_box) => inline_box,
            None => {
                // We closed the root inline box
                return;
            },
        };

        // Lay it out into a line item
        let line_item = finished_box.layout_into_line_item().into();

        // Add that line item to the new top inline box
        self.current_insertion_point().line_items.push(line_item);
    }
}

impl LineItemLayoutState {
    fn layout(&mut self, line_items: Vec<LineItem>) -> Vec<Fragment> {
        let mut fragments = Vec::new();

        for line_item in line_items {
            match line_item {
                LineItem::InlineBox(inline_box_item) => {
                    if let Some(box_fragment) = inline_box_item.layout(self) {
                        fragments.push(Fragment::Box(box_fragment));
                    }
                },
                LineItem::TextRun(text_run) => {
                    let fragment = Fragment::Text(text_run.layout(self));
                    fragments.push(fragment);
                },
            }
        }

        fragments
    }
}

impl<'box_tree> InlineBoxContainerState<'box_tree> {
    fn new(inline_box: &'box_tree InlineBox) -> Self {
        Self {
            inline_box,
            nesting_level_state: NestingLevelState::default(),
        }
    }

    fn layout_into_line_item(&mut self) -> InlineBoxItem {
        InlineBoxItem {
            style: self.inline_box.style.clone(),
            children: mem::take(&mut self.nesting_level_state.line_items),
        }
    }
}

impl TextRunItem {
    fn layout(self, state: &mut LineItemLayoutState) -> TextFragment {
        // Make the line box high enough to fit the line
        let line_height = self.metrics.size;
        if line_height > state.height {
            state.height = line_height;
        }

        let top_left = Vec2D {
            x: state.top_left.x + state.width,
            y: state.top_left.y,
        };
        let area = Rectangle {
            top_left,
            bottom_right: top_left
                + Vec2D {
                    x: self.width,
                    y: line_height,
                },
        };

        state.width += self.width;

        TextFragment::new(self.text, area, *self.style.color(), self.metrics)
    }
}

impl From<Vec<InlineLevelBox>> for InlineFormattingContext {
    fn from(elements: Vec<InlineLevelBox>) -> Self {
        Self { elements }
    }
}

impl InlineBox {
    #[inline]
    pub fn new(node: DOMPtr<dom_objects::Node>, style: ComputedStyle) -> Self {
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
                formatter.write_text(text_run.text(), "\"", "\"")?;
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

impl From<InlineBoxItem> for LineItem {
    fn from(value: InlineBoxItem) -> Self {
        Self::InlineBox(value)
    }
}

impl From<TextRunItem> for LineItem {
    fn from(value: TextRunItem) -> Self {
        Self::TextRun(value)
    }
}
