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

use super::{positioning::AbsolutelyPositionedBox, InlineFormattingContext};

/// <https://drafts.csswg.org/css2/#block-formatting>
#[derive(Clone, Default)]
pub struct BlockFormattingContext;

/// A Box that participates in a [BlockFormattingContext]
/// <https://drafts.csswg.org/css2/#block-level-boxes>
#[derive(Clone)]
pub enum BlockLevelBox {
    InFlowBlockBox(InFlowBlockBox),
    AbsolutelyPositionedBox(AbsolutelyPositionedBox),
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
        // FIXME: replaced elements

        // See https://drafts.csswg.org/css2/#blockwidth for a description of how the width is computed

        // FIXME: Consider padding and borders
        let width = self
            .style()
            .width()
            .map(|p| p.resolve_against(available_length))
            .as_ref()
            .map(|length| length.absolutize(length_resolution_context));

        let mut margin_left = resolve_margin(self.style().margin_left());
        let mut margin_right = resolve_margin(self.style().margin_right());

        let padding_left = resolve_padding(self.style().padding_left());
        let padding_right = resolve_padding(self.style().padding_right());

        let border_widths = self
            .style()
            .used_border_widths()
            .map(|side| side.absolutize(length_resolution_context));

        // Margins are treated as zero if the total width exceeds the available width
        let total_width_is_more_than_available = |width: &Pixels| {
            let total_width = margin_left.unwrap_or_default()
                + border_widths.left
                + padding_left
                + *width
                + padding_right
                + border_widths.right
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
                    - border_widths.left
                    - padding_left
                    - padding_right
                    - border_widths.right
                    - margin_right;
                (width, margin_left, margin_right)
            },
            (AutoOr::NotAuto(width), AutoOr::Auto, AutoOr::Auto) => {
                let margin_width = (containing_block.width()
                    - border_widths.left
                    - padding_left
                    - width
                    - padding_right
                    - border_widths.right)
                    / 2.;
                (width, margin_width, margin_width)
            },
            (AutoOr::NotAuto(width), AutoOr::NotAuto(margin_left), AutoOr::Auto) => {
                let margin_right = containing_block.width()
                    - margin_left
                    - border_widths.left
                    - padding_left
                    - width
                    - padding_right
                    - border_widths.right;
                (width, margin_left, margin_right)
            },
            (AutoOr::NotAuto(width), AutoOr::Auto, AutoOr::NotAuto(margin_right)) => {
                let margin_left = containing_block.width()
                    - border_widths.left
                    - padding_left
                    - width
                    - padding_right
                    - border_widths.right
                    - margin_right;
                (width, margin_left, margin_right)
            },
            (AutoOr::NotAuto(width), AutoOr::NotAuto(margin_left), AutoOr::NotAuto(_)) => {
                // The values are overconstrained
                // FIXME: If the "direction" property is "rtl", we should ignore the margin left instead
                let margin_right = containing_block.width()
                    - margin_left
                    - border_widths.left
                    - padding_left
                    - width
                    - padding_right
                    - border_widths.right;
                (width, margin_left, margin_right)
            },
        };

        // Compute the height according to https://drafts.csswg.org/css2/#normal-block
        let margin_top = resolve_margin(self.style().margin_top()).unwrap_or_default();
        let margin_bottom = resolve_margin(self.style().margin_bottom()).unwrap_or_default();

        let padding_top = resolve_padding(self.style().padding_top());
        let padding_bottom = resolve_padding(self.style().padding_bottom());

        // If the height is a percentage it is
        let height = self.style().height().flat_map(|percentage_or_length| {
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

        // The top-left corner of the content-rect
        let offset = Vec2D::new(
            margin_left + border_widths.left + padding_left,
            margin_top + border_widths.top + padding_top,
        );
        let top_left = position + offset;

        let containing_block = match height {
            AutoOr::Auto => {
                // The height of this element depends on its contents
                ContainingBlock::new(top_left, width)
            },
            AutoOr::NotAuto(height) => {
                // The height of this element is fixed, children may overflow
                ContainingBlock::new(top_left, width).with_height(height)
            },
        };

        let mut content_area_including_overflow = Rectangle::from_corners(top_left, top_left);

        // FIXME: Don't use the default font size here, it should be the font size
        // of this element instead
        let font_size = DEFAULT_FONT_SIZE;

        let length_resolution_context = length::ResolutionContext {
            font_size,
            root_font_size: length_resolution_context.root_font_size,
            viewport: length_resolution_context.viewport,
        };

        let (content_height, children) = self.contents.layout(
            top_left,
            containing_block,
            length_resolution_context,
            &mut content_area_including_overflow,
            formatting_context,
        );

        // After having looked at all the children we can now actually determine the box height
        // if it wasn't defined previously
        let height = height.unwrap_or(content_height);

        // The bottom right corner of the content area
        let bottom_right = top_left + Vec2D::new(width, height);

        let content_area = Rectangle::from_corners(top_left, bottom_right);

        let margin = Sides {
            top: margin_top,
            right: margin_right,
            bottom: margin_bottom,
            left: margin_left,
        };

        let padding = Sides {
            top: padding_top,
            right: padding_right,
            bottom: padding_bottom,
            left: padding_left,
        };

        // FIXME: This is ugly, refactor the way we tell our parent
        //        about the height of the box fragment
        let padding_area = padding.surround(content_area);
        let margin_area = margin.surround(border_widths.surround(padding_area));

        BoxFragment::new(
            self.node.clone(),
            self.style().clone(),
            margin_area,
            border_widths,
            padding_area,
            content_area_including_overflow,
            children,
        )
    }
}

impl From<InFlowBlockBox> for BlockLevelBox {
    fn from(value: InFlowBlockBox) -> Self {
        Self::InFlowBlockBox(value)
    }
}

impl From<AbsolutelyPositionedBox> for BlockLevelBox {
    fn from(value: AbsolutelyPositionedBox) -> Self {
        Self::AbsolutelyPositionedBox(value)
    }
}

impl BlockContainer {
    #[must_use]
    pub(crate) fn layout(
        &self,
        position: Vec2D<Pixels>,
        containing_block: ContainingBlock,
        ctx: length::ResolutionContext,
        content_area_including_overflow: &mut Rectangle<Pixels>,
        formatting_context: &mut BlockFormattingContext,
    ) -> (Pixels, Vec<Fragment>) {
        match &self {
            Self::BlockLevelBoxes(block_level_boxes) => {
                let mut state =
                    BlockFlowState::new(position, containing_block, ctx, formatting_context);
                for block_box in block_level_boxes {
                    state.visit_block_box(block_box);
                }
                state.finish()
            },
            Self::InlineFormattingContext(inline_formatting_context) => {
                let (fragments, height) =
                    inline_formatting_context.layout(position, containing_block, ctx);
                for fragment in &fragments {
                    content_area_including_overflow
                        .grow_to_contain(fragment.content_area_including_overflow());
                }

                (height, fragments)
            },
        }
    }
}

pub struct BlockFlowState<'box_tree, 'formatting_context> {
    block_formatting_context: &'formatting_context mut BlockFormattingContext,
    cursor: Vec2D<Pixels>,
    fragments_so_far: Vec<Fragment>,
    containing_block: ContainingBlock,
    content_area_including_overflow: Rectangle<Pixels>,
    ctx: length::ResolutionContext,
    height: Pixels,
    absolute_boxes_requiring_layout: Vec<AbsoluteBoxRequiringLayout<'box_tree>>,
}

#[derive(Clone, Copy)]
struct AbsoluteBoxRequiringLayout<'a> {
    absolute_box: &'a AbsolutelyPositionedBox,
    static_position: Vec2D<Pixels>,
    index: usize,
}

impl<'box_tree, 'formatting_context> BlockFlowState<'box_tree, 'formatting_context> {
    pub fn new(
        position: Vec2D<Pixels>,
        containing_block: ContainingBlock,
        ctx: length::ResolutionContext,
        formatting_context: &'formatting_context mut BlockFormattingContext,
    ) -> Self {
        Self {
            block_formatting_context: formatting_context,
            cursor: position,
            fragments_so_far: vec![],
            containing_block,
            content_area_including_overflow: Rectangle::from_corners(position, position),
            ctx,
            height: Pixels::ZERO,
            absolute_boxes_requiring_layout: vec![],
        }
    }

    pub fn visit_block_box(&mut self, block_box: &'box_tree BlockLevelBox) {
        match block_box {
            BlockLevelBox::InFlowBlockBox(in_flow_box) => {
                // Every block box creates exactly one box fragment
                let box_fragment = in_flow_box.fragment(
                    self.cursor,
                    self.containing_block,
                    self.ctx,
                    self.block_formatting_context,
                );

                self.content_area_including_overflow
                    .grow_to_contain(box_fragment.content_area_including_overflow());

                let box_height = box_fragment.margin_area().height();
                self.cursor.y += box_height;
                self.height += box_height;

                self.fragments_so_far.push(Fragment::Box(box_fragment));
            },
            BlockLevelBox::AbsolutelyPositionedBox(absolute_box) => {
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

    pub fn finish(self) -> (Pixels, Vec<Fragment>) {
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

        (self.height, fragments)
    }
}

impl TreeDebug for BlockLevelBox {
    fn tree_fmt(&self, formatter: &mut TreeFormatter<'_, '_>) -> fmt::Result {
        match self {
            Self::AbsolutelyPositionedBox(abs_box) => abs_box.tree_fmt(formatter),
            Self::InFlowBlockBox(block_box) => block_box.tree_fmt(formatter),
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
