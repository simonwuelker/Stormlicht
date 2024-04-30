use std::{fmt::Write, mem};

use font_metrics::FontMetrics;
use math::{Rectangle, Vec2D};

use crate::{
    css::{
        font_metrics,
        fragment_tree::{BoxFragment, Fragment, TextFragment},
        layout::{replaced::ReplacedElement, ContainingBlock, Pixels, Sides, Size},
        values::{length, FontName, VerticalAlign},
        ComputedStyle, LineBreakIterator,
    },
    dom::{dom_objects, DomPtr},
    TreeDebug, TreeFormatter,
};

/// <https://drafts.csswg.org/css2/#inline-level-boxes>
#[derive(Clone, Debug)]
pub enum InlineLevelBox {
    InlineBox(InlineBox),
    TextRun(TextRun),
    Replaced(ReplacedElement),
}

#[derive(Clone, Debug)]
pub struct TextRun {
    text: String,
    style: ComputedStyle,
}

/// <https://drafts.csswg.org/css2/#inline-box>
#[derive(Clone, Debug)]
pub struct InlineBox {
    font_size: Pixels,
    node: DomPtr<dom_objects::Node>,
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
    pub const fn new(text: String, style: ComputedStyle) -> Self {
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

    fn find_suitable_font(&self, font_size: Pixels) -> FontMetrics {
        // FIXME: Consider more than just the first specified font
        let family = match self.style().font_family().fonts()[0] {
            FontName::Family(name) => font::Family::Specific(name.to_string()),
            FontName::Generic(name) => font::Family::Generic(name.to_string()),
        };

        let properties = font::Properties {
            style: font::Style::Normal,
            weight: font::Weight::NORMAL,
            language: font::Language::English,
        };

        let font = font::SYSTEM_FONTS
            .lookup(family, properties)
            .try_load()
            .expect("Failed to load font");

        FontMetrics {
            font_face: Box::new(font),
            size: font_size,
        }
    }

    fn layout_into_line_items<'state, 'box_tree>(
        &self,
        state: &'state mut InlineFormattingContextState<'box_tree>,
    ) where
        'box_tree: 'state,
    {
        let font_metrics = self.find_suitable_font(state.current_resolution_context().font_size);

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
            let size = Size {
                width: text_line.width,
                height: font_metrics.size,
            };
            state.push_line_item(line_item, size);

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
        containing_block: ContainingBlock,
        length_resolution_context: length::ResolutionContext,
    ) -> (Vec<Fragment>, Pixels) {
        let mut state =
            InlineFormattingContextState::new(containing_block, length_resolution_context);

        state.traverse(self.elements());

        state.finish_current_line();

        if state.has_seen_relevant_content {
            (state.finished_fragments, state.y_cursor)
        } else {
            (vec![], Pixels::ZERO)
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct LineBoxUnderConstruction {
    height: Pixels,
    width: Pixels,
}

/// State of an IFC for the current nesting level
#[derive(Clone, Debug, Default)]
struct NestingLevelState<'box_tree> {
    /// The list of line items found on this nesting level so far
    line_items: Vec<LineItem<'box_tree>>,
    current_height: Pixels,
}

#[derive(Clone, Debug)]
struct InlineBoxContainerState<'box_tree> {
    inline_box: &'box_tree InlineBox,
    nesting_level_state: NestingLevelState<'box_tree>,
}

#[derive(Clone, Debug)]
struct InlineFormattingContextState<'box_tree> {
    /// Information about the line box currently being constructed
    line_box_under_construction: LineBoxUnderConstruction,

    root_nesting_level_state: NestingLevelState<'box_tree>,

    /// A stack of inline boxes that were opened within this IFC
    inline_box_stack: Vec<InlineBoxContainerState<'box_tree>>,

    containing_block: ContainingBlock,
    finished_fragments: Vec<Fragment>,
    has_seen_relevant_content: bool,

    /// The top left corner of the first line box
    y_cursor: Pixels,

    /// `true` if the current line is empty
    ///
    /// This is necessary because whitespace at the beginning of a line is removed
    at_beginning_of_line: bool,

    root_resolution_context: length::ResolutionContext,
}

#[derive(Clone, Debug)]
enum LineItem<'box_tree> {
    TextRun(TextRunItem),
    InlineBox(InlineBoxItem<'box_tree>),
    Replaced(ReplacedItem<'box_tree>),
}

#[derive(Clone, Copy, Debug)]
struct ReplacedItem<'box_tree> {
    replaced_element: &'box_tree ReplacedElement,
    size: Size<Pixels>,
}

/// A piece of text that takes up at most one line
#[derive(Clone, Debug)]
struct TextRunItem {
    metrics: FontMetrics,
    text: String,
    width: Pixels,
    style: ComputedStyle,
}

#[derive(Clone, Debug)]
struct InlineBoxItem<'box_tree> {
    style: ComputedStyle,
    children: Vec<LineItem<'box_tree>>,
    height: Pixels,
}

/// State used during conversion from [LineItems](LineItem) to [Fragments](Fragment)
///
/// This is always created for laying out line items horizontally within a single box,
/// its the inline equivalent of a [BlockFlowState](super::BlockFlowState).
///
/// However, it is guaranteed that no line breaks will occur while handling line items.
#[derive(Clone, Copy, Debug)]
struct LineItemLayoutState {
    /// The offset of the inline box from its containing inline formatting context
    position: Vec2D<Pixels>,
    current_width: Pixels,
    line_box_height: Pixels,
}

impl LineItemLayoutState {
    #[must_use]
    const fn new(position: Vec2D<Pixels>, line_box_height: Pixels) -> Self {
        Self {
            position,
            current_width: Pixels::ZERO,
            line_box_height,
        }
    }

    #[must_use]
    fn position_element(&self, height: Pixels, style: &ComputedStyle) -> Vec2D<Pixels> {
        let y_position = match style.vertical_align() {
            VerticalAlign::Baseline => self.line_box_height - height,
            other => {
                log::warn!("Implement {:?}", other);
                self.line_box_height - height
            },
        };

        self.position + Vec2D::new(self.current_width, y_position)
    }
}

impl<'box_tree> InlineBoxItem<'box_tree> {
    fn layout(self, state: &mut LineItemLayoutState) -> Option<BoxFragment> {
        // Create a nested layout state that will contain the children
        let mut nested_state =
            LineItemLayoutState::new(Vec2D::new(Pixels::ZERO, Pixels::ZERO), self.height);
        let child_fragments = nested_state.layout(self.children);

        if child_fragments.is_empty() {
            return None;
        }

        let box_position = state.position_element(self.height, &self.style);

        // FIXME: respect margins/borders for inline boxes
        let content_area = Rectangle::from_position_and_size(
            box_position,
            nested_state.current_width,
            self.height,
        );

        state.push_item(Size {
            width: nested_state.current_width,
            height: self.height,
        });

        let borders = Sides::all(Pixels::ZERO);
        let margin_area = content_area;
        let padding_area = content_area;

        let box_fragment = BoxFragment::new(
            None,
            self.style,
            margin_area,
            borders,
            padding_area,
            content_area,
            child_fragments,
        );

        Some(box_fragment)
    }
}

impl<'box_tree> InlineFormattingContextState<'box_tree> {
    fn new(
        containing_block: ContainingBlock,
        root_resolution_context: length::ResolutionContext,
    ) -> Self {
        Self {
            line_box_under_construction: LineBoxUnderConstruction::default(),
            root_nesting_level_state: NestingLevelState::default(),
            inline_box_stack: Vec::new(),
            containing_block,
            finished_fragments: Vec::new(),
            has_seen_relevant_content: false,
            y_cursor: Pixels::ZERO,
            at_beginning_of_line: true,
            root_resolution_context,
        }
    }

    fn push_line_item(&mut self, line_item: LineItem<'box_tree>, size: Size<Pixels>) {
        self.line_box_under_construction.width += size.width;
        self.has_seen_relevant_content = true;
        self.at_beginning_of_line = false;

        if self.line_box_under_construction.height < size.height {
            self.line_box_under_construction.height = size.height;
        }

        self.current_insertion_point().line_items.push(line_item);
    }

    fn remaining_width_for_line_box(&self) -> Pixels {
        self.containing_block.width() - self.line_box_under_construction.width
    }

    fn current_resolution_context(&self) -> length::ResolutionContext {
        let innermost_font_size = self
            .inline_box_stack
            .last()
            .map(|container| container.inline_box.font_size);

        if let Some(font_size) = innermost_font_size {
            self.root_resolution_context.with_font_size(font_size)
        } else {
            self.root_resolution_context
        }
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
                InlineLevelBox::Replaced(replaced_element) => {
                    let size = replaced_element.used_size_if_it_was_inline(
                        self.containing_block,
                        self.root_resolution_context,
                    );
                    let replaced_item = ReplacedItem {
                        replaced_element,
                        size,
                    };
                    self.push_line_item(replaced_item.into(), size);
                },
            }
        }
    }

    fn finish_current_line<'a>(&'a mut self)
    where
        'box_tree: 'a,
    {
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

        let mut layout_state = LineItemLayoutState::new(
            Vec2D::new(Pixels::ZERO, self.y_cursor),
            self.line_box_under_construction.height,
        );
        self.finished_fragments
            .extend(layout_state.layout(items_on_this_line));

        self.y_cursor += self.line_box_under_construction.height;

        // Prepare for a new line
        self.line_box_under_construction = LineBoxUnderConstruction {
            width: Pixels::ZERO,
            height: Pixels::ZERO,
        };

        self.at_beginning_of_line = true;
    }

    fn current_insertion_point(&mut self) -> &mut NestingLevelState<'box_tree> {
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
        let line_item = finished_box.layout_into_line_item();

        // Add that line item to the new top inline box
        let current_insertion_point = self.current_insertion_point();
        current_insertion_point.current_height =
            current_insertion_point.current_height.max(line_item.height);
        current_insertion_point.line_items.push(line_item.into());
    }
}

impl<'box_tree> LineItemLayoutState {
    /// Returns all the fragments within a line box
    fn layout(&mut self, line_items: Vec<LineItem<'box_tree>>) -> Vec<Fragment> {
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
                LineItem::Replaced(replaced_item) => {
                    let fragment = replaced_item.layout(self);
                    fragments.push(fragment);
                },
            }
        }

        fragments
    }

    fn push_item(&mut self, size: Size<Pixels>) {
        self.current_width += size.width;
    }
}

impl<'box_tree> InlineBoxContainerState<'box_tree> {
    #[must_use]
    fn new(inline_box: &'box_tree InlineBox) -> Self {
        Self {
            inline_box,
            nesting_level_state: NestingLevelState::default(),
        }
    }

    #[must_use]
    fn layout_into_line_item<'a>(&'a mut self) -> InlineBoxItem<'box_tree>
    where
        'box_tree: 'a,
    {
        InlineBoxItem {
            style: self.inline_box.style.clone(),
            children: mem::take(&mut self.nesting_level_state.line_items),
            height: self.nesting_level_state.current_height,
        }
    }
}

impl<'box_tree> ReplacedItem<'box_tree> {
    #[must_use]
    fn layout(self, state: &mut LineItemLayoutState) -> Fragment {
        state.push_item(self.size);

        self.replaced_element
            .content()
            .create_fragment(state.position, self.size)
    }
}

impl TextRunItem {
    #[must_use]
    fn layout(self, state: &mut LineItemLayoutState) -> TextFragment {
        // Make the line box high enough to fit the line
        let line_height = self.metrics.size;

        let position = state.position_element(self.metrics.size, &self.style);
        let area = Rectangle::from_position_and_size(position, self.width, line_height);

        state.push_item(Size {
            width: self.width,
            height: line_height,
        });

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
    #[must_use]
    pub fn new(node: DomPtr<dom_objects::Node>, style: ComputedStyle, font_size: Pixels) -> Self {
        Self {
            node,
            style,
            font_size,
            contents: Vec::new(),
        }
    }

    #[inline]
    #[must_use]
    pub const fn font_size(&self) -> Pixels {
        self.font_size
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
    #[must_use]
    pub fn split_off(&self) -> Self {
        Self {
            node: self.node.clone(),
            style: self.style.clone(),
            contents: vec![],
            font_size: self.font_size,
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
            Self::Replaced(_) => {
                formatter.indent()?;
                writeln!(formatter, "Replaced Element")?;
            },
        }
        Ok(())
    }
}

impl<'box_tree> From<InlineBoxItem<'box_tree>> for LineItem<'box_tree> {
    fn from(value: InlineBoxItem<'box_tree>) -> Self {
        Self::InlineBox(value)
    }
}

impl<'box_tree> From<TextRunItem> for LineItem<'box_tree> {
    fn from(value: TextRunItem) -> Self {
        Self::TextRun(value)
    }
}

impl<'box_tree> From<ReplacedItem<'box_tree>> for LineItem<'box_tree> {
    fn from(value: ReplacedItem<'box_tree>) -> Self {
        Self::Replaced(value)
    }
}
