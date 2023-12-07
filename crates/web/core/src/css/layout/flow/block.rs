use std::{fmt, fmt::Write};

use math::{Rectangle, Vec2D};

use crate::{
    css::{
        font_metrics::DEFAULT_FONT_SIZE,
        fragment_tree::{BoxFragment, Fragment},
        layout::{ContainingBlock, Pixels, Sides},
        values::{self, length, AutoOr, Length, PercentageOr},
        ComputedStyle,
    },
    dom::{dom_objects, DomPtr},
    TreeDebug, TreeFormatter,
};

use super::{
    positioning::AbsolutelyPositionedBox, FloatContext, FloatingBox, InlineFormattingContext,
};

/// <https://drafts.csswg.org/css2/#block-formatting>
///
/// Holds state about collapsible margins and floating elements.
#[derive(Clone)]
pub struct BlockFormattingContext {
    last_margin: Pixels,
    float_context: FloatContext,
}

impl BlockFormattingContext {
    #[must_use]
    pub fn new(containing_block: ContainingBlock) -> Self {
        Self {
            last_margin: Pixels::ZERO,
            float_context: FloatContext::new(containing_block),
        }
    }

    fn prevent_margin_collapse(&mut self) {
        self.last_margin = Pixels::ZERO;
    }

    fn get_collapsed_margin(&mut self, specified_margin: Pixels) -> Pixels {
        if specified_margin <= self.last_margin {
            // The new margin fully collapses into the previous one
            Pixels::ZERO
        } else {
            let used_margin = specified_margin - self.last_margin;
            self.last_margin = specified_margin;
            used_margin
        }
    }
}

/// A Box that participates in a [BlockFormattingContext]
/// <https://drafts.csswg.org/css2/#block-level-boxes>
#[derive(Clone)]
pub enum BlockLevelBox {
    Floating(FloatingBox),
    InFlow(InFlowBlockBox),
    AbsolutelyPositioned(AbsolutelyPositionedBox),
}
#[derive(Clone)]
pub struct InFlowBlockBox {
    style: ComputedStyle,

    /// The DOM element that produced this box.
    /// Some boxes might not correspond to a DOM node,
    /// for example anonymous block boxes
    node: Option<DomPtr<dom_objects::Node>>,

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

impl InFlowBlockBox {
    #[must_use]
    pub const fn new(
        style: ComputedStyle,
        node: Option<DomPtr<dom_objects::Node>>,
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
    pub const fn style(&self) -> &ComputedStyle {
        &self.style
    }

    #[must_use]
    pub fn create_anonymous_box(contents: BlockContainer, style: ComputedStyle) -> Self {
        Self {
            style,
            node: None,
            contents,
        }
    }

    /// Compute layout for this block box, turning it into a fragment
    ///
    /// The `position` parameter describes the top-left corner of the parents
    /// content rect.
    fn fragment(
        &self,
        position: Vec2D<Pixels>,
        containing_block: ContainingBlock,
        length_resolution_context: length::ResolutionContext,
        formatting_context: &mut BlockFormattingContext,
    ) -> BoxFragment {
        let mut dimensions =
            BlockDimensions::compute(self.style(), containing_block, length_resolution_context);

        // Possibly collapse top margin
        dimensions.margin.top = formatting_context.get_collapsed_margin(dimensions.margin.top);

        // FIXME: Don't use the default font size here, it should be the font size
        // of this element instead
        let font_size = DEFAULT_FONT_SIZE;

        let length_resolution_context = length_resolution_context.with_font_size(font_size);

        // The top-left corner of the content-rect
        let top_left = position + dimensions.content_offset();

        // Prevent margin-collapse of our top margin with the top margin of the
        // first in-flow child if there is a top border or top padding on this element
        if dimensions.border.top != Pixels::ZERO || dimensions.padding.top != Pixels::ZERO {
            formatting_context.prevent_margin_collapse();
        }

        let content_info = self.contents.layout(
            dimensions.as_containing_block(),
            length_resolution_context,
            formatting_context,
        );

        // If the content did not contain any in-flow elements *but* it has a nonzero
        // height anyways then it does prevent the top and bottom margins from collapsing
        if !content_info.has_in_flow_content
            && dimensions.height.is_not_auto_and(|&l| l != Pixels::ZERO)
        {
            formatting_context.prevent_margin_collapse();
        }

        // Prevent margin-collapse of our bottom margin with the bottom margin of the
        // last in-flow child if there is a bottom border or bottom padding on this element
        if dimensions.border.bottom != Pixels::ZERO || dimensions.padding.bottom != Pixels::ZERO {
            formatting_context.prevent_margin_collapse();
        }

        dimensions.margin.bottom =
            formatting_context.get_collapsed_margin(dimensions.margin.bottom);

        // After having looked at all the children we can now actually determine the box height
        // if it wasn't defined previously
        let height = dimensions.height.unwrap_or(content_info.height);

        // The bottom right corner of the content area
        let bottom_right = top_left + Vec2D::new(dimensions.width, height);

        let content_area = Rectangle::from_corners(top_left, bottom_right);

        // FIXME: This is ugly, refactor the way we tell our parent
        //        about the height of the box fragment
        let padding_area = dimensions.padding.surround(content_area);
        let margin_area = dimensions
            .margin
            .surround(dimensions.border.surround(padding_area));

        BoxFragment::new(
            self.node.clone(),
            self.style().clone(),
            margin_area,
            dimensions.border,
            padding_area,
            content_area,
            content_info.fragments,
        )
    }
}

impl From<FloatingBox> for BlockLevelBox {
    fn from(value: FloatingBox) -> Self {
        Self::Floating(value)
    }
}

impl From<InFlowBlockBox> for BlockLevelBox {
    fn from(value: InFlowBlockBox) -> Self {
        Self::InFlow(value)
    }
}

impl From<AbsolutelyPositionedBox> for BlockLevelBox {
    fn from(value: AbsolutelyPositionedBox) -> Self {
        Self::AbsolutelyPositioned(value)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ContentLayoutInfo {
    pub height: Pixels,
    pub fragments: Vec<Fragment>,
    pub has_in_flow_content: bool,
}

impl BlockContainer {
    #[must_use]
    pub(crate) fn layout(
        &self,
        containing_block: ContainingBlock,
        ctx: length::ResolutionContext,
        formatting_context: &mut BlockFormattingContext,
    ) -> ContentLayoutInfo {
        match &self {
            Self::BlockLevelBoxes(block_level_boxes) => {
                let mut state = BlockFlowState::new(containing_block, ctx, formatting_context);
                for block_box in block_level_boxes {
                    state.visit_block_box(block_box);
                }
                state.finish()
            },
            Self::InlineFormattingContext(inline_formatting_context) => {
                // Margins cannot collapse across inline formatting contexts
                // FIXME: Zero-height line boxes do not prevent margin collapse
                //        https://drafts.csswg.org/css2/#inline-formatting
                formatting_context.prevent_margin_collapse();

                let (fragments, height) = inline_formatting_context.layout(containing_block, ctx);

                ContentLayoutInfo {
                    height,
                    fragments,
                    has_in_flow_content: true,
                }
            },
        }
    }
}

pub struct BlockFlowState<'box_tree, 'formatting_context> {
    block_formatting_context: &'formatting_context mut BlockFormattingContext,
    cursor: Vec2D<Pixels>,
    fragments_so_far: Vec<Fragment>,
    containing_block: ContainingBlock,
    ctx: length::ResolutionContext,
    height: Pixels,
    absolute_boxes_requiring_layout: Vec<AbsoluteBoxRequiringLayout<'box_tree>>,
    has_in_flow_content: bool,
}

#[derive(Clone, Copy)]
struct AbsoluteBoxRequiringLayout<'a> {
    absolute_box: &'a AbsolutelyPositionedBox,
    static_position: Vec2D<Pixels>,
    index: usize,
}

impl<'box_tree, 'formatting_context> BlockFlowState<'box_tree, 'formatting_context> {
    pub fn new(
        containing_block: ContainingBlock,
        ctx: length::ResolutionContext,
        formatting_context: &'formatting_context mut BlockFormattingContext,
    ) -> Self {
        Self {
            cursor: Vec2D::new(Pixels::ZERO, Pixels::ZERO),
            block_formatting_context: formatting_context,
            fragments_so_far: vec![],
            containing_block,
            ctx,
            height: Pixels::ZERO,
            absolute_boxes_requiring_layout: vec![],
            has_in_flow_content: false,
        }
    }

    pub fn visit_block_box(&mut self, block_box: &'box_tree BlockLevelBox) {
        match block_box {
            BlockLevelBox::Floating(float_box) => {
                // Floats are placed at or below the flow position
                self.block_formatting_context
                    .float_context
                    .lower_float_ceiling(self.cursor.y);

                let box_fragment = float_box.layout(
                    self.containing_block,
                    self.ctx,
                    &mut self.block_formatting_context.float_context,
                );

                self.fragments_so_far.push(box_fragment.into())
            },
            BlockLevelBox::InFlow(in_flow_box) => {
                // Every block box creates exactly one box fragment
                let box_fragment = in_flow_box.fragment(
                    self.cursor,
                    self.containing_block,
                    self.ctx,
                    self.block_formatting_context,
                );

                let box_height = box_fragment.margin_area().height();
                self.cursor.y += box_height;
                self.height += box_height;

                self.fragments_so_far.push(Fragment::Box(box_fragment));
            },
            BlockLevelBox::AbsolutelyPositioned(absolute_box) => {
                // Absolutely positioned boxes cannot be laid out during the initial pass,
                // as their positioning requires the size of the containing block to be known.
                //
                // However, they should still be painted in tree-order.
                // To accomodate this, we keep track of the absolute boxes we found during the first
                // pass and later insert the fragments at the correct position once the
                // size of the containing block is known.
                self.absolute_boxes_requiring_layout
                    .push(AbsoluteBoxRequiringLayout {
                        absolute_box,
                        static_position: self.cursor,
                        index: self.fragments_so_far.len(),
                    });
            },
        }
    }

    pub fn finish(self) -> ContentLayoutInfo {
        // Now that we have processed all in-flow elements, we can layout absolutely positioned
        // elements.
        let mut fragments = self.fragments_so_far;
        let definite_containing_block = self.containing_block.make_definite(self.height);

        for task in self.absolute_boxes_requiring_layout {
            let fragment =
                task.absolute_box
                    .layout(definite_containing_block, self.ctx, task.static_position);
            fragments.insert(task.index, fragment.into());
        }

        ContentLayoutInfo {
            height: self.height,
            fragments,
            has_in_flow_content: self.has_in_flow_content,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct BlockDimensions {
    margin: Sides<Pixels>,
    padding: Sides<Pixels>,
    border: Sides<Pixels>,
    width: Pixels,
    height: AutoOr<Pixels>,
}

impl BlockDimensions {
    /// Compute dimensions for normal, non replaced block elements
    ///
    /// The relevant parts of the specification are:
    /// * https://drafts.csswg.org/css2/#blockwidth
    /// * https://drafts.csswg.org/css2/#normal-block
    ///
    /// This method does **not** layout the blocks contents nor does it perform margin-collapsing.
    #[must_use]
    fn compute(
        style: &ComputedStyle,
        containing_block: ContainingBlock,
        length_resolution_context: length::ResolutionContext,
    ) -> Self {
        // FIXME: replaced elements
        let available_length = Length::pixels(containing_block.width());

        let resolve_margin = |margin: &values::Margin| {
            margin
                .map(|p| p.resolve_against(available_length))
                .as_ref()
                .map(|length| length.absolutize(length_resolution_context))
        };

        let resolve_padding = |padding: &values::Padding| {
            padding
                .resolve_against(available_length)
                .absolutize(length_resolution_context)
        };

        let padding = Sides {
            top: resolve_padding(style.padding_top()),
            right: resolve_padding(style.padding_right()),
            bottom: resolve_padding(style.padding_bottom()),
            left: resolve_padding(style.padding_left()),
        };

        let border = style
            .used_border_widths()
            .map(|side| side.absolutize(length_resolution_context));

        // See https://drafts.csswg.org/css2/#blockwidth for a description of how the width is computed
        let width = style
            .width()
            .map(|p| p.resolve_against(available_length))
            .as_ref()
            .map(|length| length.absolutize(length_resolution_context));

        let mut margin_left = resolve_margin(style.margin_left());
        let mut margin_right = resolve_margin(style.margin_right());

        // Margins are treated as zero if the total width exceeds the available width
        let total_width_is_more_than_available = |width: &Pixels| {
            let total_width = margin_left.unwrap_or_default()
                + border.horizontal_sum()
                + padding.horizontal_sum()
                + *width
                + margin_right.unwrap_or_default();
            total_width > containing_block.width()
        };
        if width.is_not_auto_and(total_width_is_more_than_available) {
            margin_left = margin_left.or(AutoOr::NotAuto(Pixels::ZERO));
            margin_right = margin_right.or(AutoOr::NotAuto(Pixels::ZERO));
        }

        // If there is exactly one value specified as auto, its used value follows from the equality.
        let (width, margin_left, margin_right) = match (width, margin_left, margin_right) {
            (AutoOr::Auto, margin_left, margin_right) => {
                // If width is set to auto, any other auto values become 0 and width follows from the resulting equality.
                let margin_left: Pixels = margin_left.unwrap_or(Pixels::ZERO);
                let margin_right = margin_right.unwrap_or(Pixels::ZERO);
                let width = containing_block.width()
                    - margin_left
                    - border.horizontal_sum()
                    - padding.horizontal_sum()
                    - margin_right;
                (width, margin_left, margin_right)
            },
            (AutoOr::NotAuto(width), AutoOr::Auto, AutoOr::Auto) => {
                let margin_width =
                    (containing_block.width() - border.horizontal_sum() - padding.horizontal_sum())
                        / 2.;
                (width, margin_width, margin_width)
            },
            (AutoOr::NotAuto(width), AutoOr::NotAuto(margin_left), AutoOr::Auto) => {
                let margin_right = containing_block.width()
                    - margin_left
                    - border.horizontal_sum()
                    - padding.horizontal_sum();
                (width, margin_left, margin_right)
            },
            (AutoOr::NotAuto(width), AutoOr::Auto, AutoOr::NotAuto(margin_right)) => {
                let margin_left = containing_block.width()
                    - border.horizontal_sum()
                    - padding.horizontal_sum()
                    - width
                    - margin_right;
                (width, margin_left, margin_right)
            },
            (AutoOr::NotAuto(width), AutoOr::NotAuto(margin_left), AutoOr::NotAuto(_)) => {
                // The values are overconstrained
                // FIXME: If the "direction" property is "rtl", we should ignore the margin left instead
                let margin_right = containing_block.width()
                    - margin_left
                    - border.horizontal_sum()
                    - padding.horizontal_sum()
                    - width;
                (width, margin_left, margin_right)
            },
        };

        // Compute the height according to https://drafts.csswg.org/css2/#normal-block
        // If the height is a percentage it is
        let height = style.height().flat_map(|percentage_or_length| {
            match percentage_or_length {
                PercentageOr::Percentage(percentage) => {
                    if let Some(available_height) = containing_block.height() {
                        AutoOr::NotAuto(available_height * percentage.as_fraction())
                    } else {
                        // If the value is a percentage but the length of the containing block is not
                        // yet determined, the value should be treated as auto.
                        // (https://drafts.csswg.org/css2/#the-height-property)
                        AutoOr::Auto
                    }
                },
                PercentageOr::NotPercentage(length) => {
                    AutoOr::NotAuto(length.absolutize(length_resolution_context))
                },
            }
        });

        let margin = Sides {
            top: resolve_margin(style.margin_top()).unwrap_or_default(),
            right: margin_right,
            bottom: resolve_margin(style.margin_bottom()).unwrap_or_default(),
            left: margin_left,
        };

        Self {
            margin,
            padding,
            border,
            width,
            height,
        }
    }

    /// Return the offset of the top-left corner of the content area from the top-left corner
    /// of the margin area
    #[must_use]
    fn content_offset(&self) -> Vec2D<Pixels> {
        Vec2D::new(
            self.margin.left + self.border.left + self.padding.left,
            self.margin.top + self.border.top + self.margin.top,
        )
    }

    #[must_use]
    fn as_containing_block(&self) -> ContainingBlock {
        ContainingBlock {
            width: self.width,
            height: self.height.into_option(),
        }
    }
}

impl TreeDebug for BlockLevelBox {
    fn tree_fmt(&self, formatter: &mut TreeFormatter<'_, '_>) -> fmt::Result {
        match self {
            Self::Floating(float_box) => float_box.tree_fmt(formatter),
            Self::AbsolutelyPositioned(abs_box) => abs_box.tree_fmt(formatter),
            Self::InFlow(block_box) => block_box.tree_fmt(formatter),
        }
    }
}

impl TreeDebug for InFlowBlockBox {
    fn tree_fmt(&self, formatter: &mut TreeFormatter<'_, '_>) -> std::fmt::Result {
        formatter.indent()?;
        write!(formatter, "Block Box")?;
        if let Some(node) = &self.node {
            writeln!(formatter, " ({:?})", node.underlying_type())?;
        } else {
            writeln!(formatter, " (anonymous)")?;
        }

        formatter.increase_indent();
        self.contents.tree_fmt(formatter)?;
        formatter.decrease_indent();
        Ok(())
    }
}

impl TreeDebug for BlockContainer {
    fn tree_fmt(&self, formatter: &mut TreeFormatter<'_, '_>) -> fmt::Result {
        match &self {
            Self::BlockLevelBoxes(block_level_boxes) => {
                for block_box in block_level_boxes {
                    block_box.tree_fmt(formatter)?;
                }
                Ok(())
            },
            Self::InlineFormattingContext(inline_formatting_context) => {
                inline_formatting_context.tree_fmt(formatter)
            },
        }
    }
}
