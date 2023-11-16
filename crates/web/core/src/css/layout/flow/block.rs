use std::{fmt, fmt::Write};

use math::{Rectangle, Vec2D};

use crate::{
    css::{
        font_metrics::DEFAULT_FONT_SIZE,
        fragment_tree::{BoxFragment, Fragment, FragmentTree},
        layout::{ContainingBlock, Pixels, Sides, Size},
        values::{self, length, AutoOr, Length, PercentageOr},
        ComputedStyle, StyleComputer,
    },
    dom::{dom_objects, DomPtr},
    TreeDebug, TreeFormatter,
};

use super::{BoxTreeBuilder, InlineFormattingContext};

/// <https://drafts.csswg.org/css2/#block-formatting>
#[derive(Clone)]
pub struct BlockFormattingContext {
    contents: Vec<BlockLevelBox>,
}

/// A Box that participates in a [BlockFormattingContext]
/// <https://drafts.csswg.org/css2/#block-level-boxes>
#[derive(Clone)]
pub struct BlockLevelBox {
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

impl BlockFormattingContext {
    pub fn root(
        document: DomPtr<dom_objects::Document>,
        style_computer: StyleComputer<'_>,
    ) -> Self {
        let html = document
            .borrow()
            .children()
            .last()
            .expect("no root element found")
            .try_into_type::<dom_objects::HtmlHtmlElement>()
            .expect("expected root element to be html element");

        let document_style = style_computer
            .get_computed_style(DomPtr::clone(&html).upcast(), &ComputedStyle::default());

        let contents = BoxTreeBuilder::build(
            DomPtr::clone(&html).upcast(),
            style_computer,
            &document_style,
        );

        let root = BlockLevelBox {
            style: document_style,
            contents,
            node: Some(html.upcast()),
        };

        vec![root].into()
    }

    pub fn fragment(&self, viewport: Size<Pixels>) -> FragmentTree {
        let position = Vec2D::new(Pixels::ZERO, Pixels::ZERO);
        let length_resolution_context = length::ResolutionContext {
            font_size: DEFAULT_FONT_SIZE,
            root_font_size: DEFAULT_FONT_SIZE,
            viewport,
        };

        let mut cursor_position = position;

        let mut root_fragments = vec![];
        let containing_block = ContainingBlock::new(viewport.width).with_height(viewport.height);
        for element in &self.contents {
            let box_fragment =
                element.fragment(cursor_position, containing_block, length_resolution_context);
            cursor_position.y += box_fragment.margin_area().height();
            root_fragments.push(Fragment::Box(box_fragment));
        }

        FragmentTree::new(root_fragments)
    }
}

impl From<Vec<BlockLevelBox>> for BlockFormattingContext {
    fn from(contents: Vec<BlockLevelBox>) -> Self {
        Self { contents }
    }
}

impl BlockLevelBox {
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
    ) -> BoxFragment {
        let available_length = Length::pixels(containing_block.width());

        let resolve_margin = |margin: &values::Margin| {
            margin
                .map(|p| p.resolve_against(available_length))
                .as_ref()
                .map(|length| length.absolutize(length_resolution_context))
        };

        let resolve_border_width =
            |border: &values::LineWidth| border.length().absolutize(length_resolution_context);

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

        let border_left = if self.style().border_left_style().is_none() {
            Pixels::ZERO
        } else {
            resolve_border_width(self.style().border_left_width())
        };

        let border_right = if self.style().border_right_style().is_none() {
            Pixels::ZERO
        } else {
            resolve_border_width(self.style().border_right_width())
        };

        // Margins are treated as zero if the total width exceeds the available width
        let total_width_is_more_than_available = |width: &Pixels| {
            let total_width = margin_left.unwrap_or_default()
                + border_left
                + padding_left
                + *width
                + padding_right
                + border_right
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
                    - border_left
                    - padding_left
                    - padding_right
                    - border_right
                    - margin_right;
                (width, margin_left, margin_right)
            },
            (AutoOr::NotAuto(width), AutoOr::Auto, AutoOr::Auto) => {
                let margin_width = (containing_block.width()
                    - border_left
                    - padding_left
                    - width
                    - padding_right
                    - border_right)
                    / 2.;
                (width, margin_width, margin_width)
            },
            (AutoOr::NotAuto(width), AutoOr::NotAuto(margin_left), AutoOr::Auto) => {
                let margin_right = containing_block.width()
                    - margin_left
                    - border_left
                    - padding_left
                    - width
                    - padding_right
                    - border_right;
                (width, margin_left, margin_right)
            },
            (AutoOr::NotAuto(width), AutoOr::Auto, AutoOr::NotAuto(margin_right)) => {
                let margin_left = containing_block.width()
                    - border_left
                    - padding_left
                    - width
                    - padding_right
                    - border_right
                    - margin_right;
                (width, margin_left, margin_right)
            },
            (AutoOr::NotAuto(width), AutoOr::NotAuto(margin_left), AutoOr::NotAuto(_)) => {
                // The values are overconstrained
                // FIXME: If the "direction" property is "rtl", we should ignore the margin left instead
                let margin_right = containing_block.width()
                    - margin_left
                    - border_left
                    - padding_left
                    - width
                    - padding_right
                    - border_right;
                (width, margin_left, margin_right)
            },
        };

        // Compute the height according to https://drafts.csswg.org/css2/#normal-block
        let margin_top = resolve_margin(self.style().margin_top()).unwrap_or_default();
        let margin_bottom = resolve_margin(self.style().margin_bottom()).unwrap_or_default();

        let padding_top = resolve_padding(self.style().padding_top());
        let padding_bottom = resolve_padding(self.style().padding_bottom());

        let border_top = if self.style().border_top_style().is_none() {
            Pixels::ZERO
        } else {
            resolve_border_width(self.style().border_top_width())
        };

        let border_bottom = if self.style().border_bottom_style().is_none() {
            Pixels::ZERO
        } else {
            resolve_border_width(self.style().border_bottom_width())
        };

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

        let containing_block = match height {
            AutoOr::Auto => {
                // The height of this element depends on its contents
                ContainingBlock::new(width)
            },
            AutoOr::NotAuto(height) => {
                // The height of this element is fixed, children may overflow
                ContainingBlock::new(width).with_height(height)
            },
        };

        // The top-left corner of the content-rect
        let offset = Vec2D::new(
            margin_left + border_left + padding_left,
            margin_top + border_top + padding_top,
        );
        let top_left = position + offset;

        let mut content_area_including_overflow = Rectangle::from_corners(top_left, top_left);

        // FIXME: Don't use the default font size here, it should be the font size
        // of this element instead
        let font_size = DEFAULT_FONT_SIZE;

        let length_resolution_context = length::ResolutionContext {
            font_size,
            root_font_size: length_resolution_context.root_font_size,
            viewport: length_resolution_context.viewport,
        };

        let (content_height, children) = match &self.contents {
            BlockContainer::BlockLevelBoxes(block_level_boxes) => {
                let mut state = BlockFormattingContextState::new(
                    top_left,
                    containing_block,
                    length_resolution_context,
                );

                for block_box in block_level_boxes {
                    state.visit_block_box(block_box);
                }

                state.finish()
            },
            BlockContainer::InlineFormattingContext(inline_formatting_context) => {
                let (fragments, content_height) = inline_formatting_context.layout(
                    top_left,
                    containing_block,
                    length_resolution_context,
                );
                for fragment in &fragments {
                    content_area_including_overflow
                        .grow_to_contain(fragment.content_area_including_overflow());
                }

                let height = padding_top + content_height + padding_bottom;

                (height, fragments)
            },
        };

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
        let borders = Sides {
            top: border_top,
            right: border_right,
            bottom: border_bottom,
            left: border_left,
        };

        let padding_area = padding.surround(content_area);
        let margin_area = margin.surround(borders.surround(padding_area));

        BoxFragment::new(
            self.node.clone(),
            self.style().clone(),
            margin_area,
            borders,
            padding_area,
            content_area_including_overflow,
            children,
        )
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

#[derive(Clone, Debug)]
struct BlockFormattingContextState {
    cursor: Vec2D<Pixels>,
    fragments_so_far: Vec<Fragment>,
    containing_block: ContainingBlock,
    content_area_including_overflow: Rectangle<Pixels>,
    ctx: length::ResolutionContext,
    height: Pixels,
}

impl BlockFormattingContextState {
    fn new(
        position: Vec2D<Pixels>,
        containing_block: ContainingBlock,
        ctx: length::ResolutionContext,
    ) -> Self {
        Self {
            cursor: position,
            fragments_so_far: vec![],
            containing_block,
            content_area_including_overflow: Rectangle::from_corners(position, position),
            ctx,
            height: Pixels::ZERO,
        }
    }

    fn visit_block_box(&mut self, block_box: &BlockLevelBox) {
        // Every block box creates exactly one box fragment
        let box_fragment = block_box.fragment(self.cursor, self.containing_block, self.ctx);

        self.content_area_including_overflow
            .grow_to_contain(box_fragment.content_area_including_overflow());

        let box_height = box_fragment.margin_area().height();
        self.cursor.y += box_height;
        self.height += box_height;

        self.fragments_so_far.push(Fragment::Box(box_fragment));
    }

    fn finish(self) -> (Pixels, Vec<Fragment>) {
        (self.height, self.fragments_so_far)
    }
}
