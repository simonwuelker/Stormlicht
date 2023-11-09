use std::{fmt, fmt::Write};

use math::{Rectangle, Vec2D};

use crate::{
    css::{
        font_metrics::DEFAULT_FONT_SIZE,
        fragment_tree::{BoxFragment, Fragment, FragmentTree},
        layout::{CSSPixels, ContainingBlock, Sides, Size},
        values::{length, AutoOr, Length, PercentageOr},
        ComputedStyle, StyleComputer,
    },
    dom::{dom_objects, DOMPtr},
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
        let document_style = style_computer
            .get_computed_style(document.clone().into_type(), &ComputedStyle::default());

        let contents = BoxTreeBuilder::build(document.clone(), style_computer, &document_style);

        let root = BlockLevelBox {
            style: document_style,
            contents,
            node: Some(document),
        };

        vec![root].into()
    }

    pub fn fragment(&self, viewport: Size<CSSPixels>) -> FragmentTree {
        let position = Vec2D {
            x: CSSPixels::ZERO,
            y: CSSPixels::ZERO,
        };
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
            cursor_position.y += box_fragment.outer_area().height();
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
    pub fn new(
        style: ComputedStyle,
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
    pub fn style(&self) -> &ComputedStyle {
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

    fn fragment(
        &self,
        position: Vec2D<CSSPixels>,
        containing_block: ContainingBlock,
        length_resolution_context: length::ResolutionContext,
    ) -> BoxFragment {
        // FIXME: replaced elements

        // See https://drafts.csswg.org/css2/#blockwidth for a description of how the width is computed

        // FIXME: Consider padding and borders
        let available_length = Length::pixels(containing_block.width());
        let width = self
            .style()
            .width()
            .map(|p| p.resolve_against(available_length))
            .as_ref()
            .map(|length| length.absolutize(length_resolution_context));

        let mut margin_left = self
            .style()
            .margin_left()
            .map(|p| p.resolve_against(available_length))
            .as_ref()
            .map(|length| length.absolutize(length_resolution_context));

        let mut margin_right = self
            .style()
            .margin_right()
            .map(|p| p.resolve_against(available_length))
            .as_ref()
            .map(|length| length.absolutize(length_resolution_context));

        let padding_left = self
            .style()
            .padding_left()
            .resolve_against(available_length)
            .absolutize(length_resolution_context);

        let padding_right = self
            .style()
            .padding_right()
            .resolve_against(available_length)
            .absolutize(length_resolution_context);

        // Margins are treated as zero if the total width exceeds the available width
        let total_width_is_more_than_available = |width: &CSSPixels| {
            let total_width = margin_left.unwrap_or_default()
                + padding_left
                + *width
                + padding_right
                + margin_right.unwrap_or_default();
            total_width > containing_block.width()
        };
        if width.is_not_auto_and(total_width_is_more_than_available) {
            margin_left = margin_left.or(AutoOr::NotAuto(CSSPixels::ZERO));
            margin_right = margin_right.or(AutoOr::NotAuto(CSSPixels::ZERO));
        }

        // If there is exactly one value specified as auto, its used value follows from the equality.
        let (width, margin_left, margin_right) = match (width, margin_left, margin_right) {
            (AutoOr::Auto, margin_left, margin_right) => {
                // If width is set to auto, any other auto values become 0 and width follows from the resulting equality.
                let margin_left: CSSPixels = margin_left.unwrap_or(CSSPixels::ZERO);
                let margin_right = margin_right.unwrap_or(CSSPixels::ZERO);
                let width = containing_block.width() - margin_left - margin_right;
                (width, margin_left, margin_right)
            },
            (AutoOr::NotAuto(width), AutoOr::Auto, AutoOr::Auto) => {
                let margin_width =
                    (containing_block.width() - padding_left - width - padding_right) / 2.;
                (width, margin_width, margin_width)
            },
            (AutoOr::NotAuto(width), AutoOr::NotAuto(margin_left), AutoOr::Auto) => {
                let margin_right =
                    containing_block.width() - margin_left - padding_left - width - padding_right;
                (width, margin_left, margin_right)
            },
            (AutoOr::NotAuto(width), AutoOr::Auto, AutoOr::NotAuto(margin_right)) => {
                let margin_left =
                    containing_block.width() - padding_left - width - padding_right - margin_right;
                (width, margin_left, margin_right)
            },
            (AutoOr::NotAuto(width), AutoOr::NotAuto(margin_left), AutoOr::NotAuto(_)) => {
                // The values are overconstrained
                // FIXME: If the "direction" property is "rtl", we should ignore the margin left instead
                let margin_right =
                    containing_block.width() - margin_left - padding_left - width - padding_right;
                (width, margin_left, margin_right)
            },
        };

        // Compute the height according to https://drafts.csswg.org/css2/#normal-block
        let margin_top = self
            .style()
            .margin_top()
            .map(|p| p.resolve_against(available_length))
            .as_ref()
            .map(|length| length.absolutize(length_resolution_context))
            .unwrap_or_default();

        let margin_bottom = self
            .style()
            .margin_bottom()
            .map(|p| p.resolve_against(available_length))
            .as_ref()
            .map(|length| length.absolutize(length_resolution_context))
            .unwrap_or_default();

        let padding_top = self
            .style()
            .padding_top()
            .resolve_against(available_length)
            .absolutize(length_resolution_context);

        let padding_bottom = self
            .style()
            .padding_bottom()
            .resolve_against(available_length)
            .absolutize(length_resolution_context);

        // FIXME:
        // * Consider height: auto
        // * Resolve percentages against height (what is it?), not widht
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

        let mut children = vec![];

        let content_width = width - padding_left - padding_right;
        let containing_block = match height {
            AutoOr::Auto => {
                // The height of this element depends on its contents
                ContainingBlock::new(content_width)
            },
            AutoOr::NotAuto(height) => {
                // The height of this element is fixed, children may overflow
                let content_height = height - padding_top - padding_bottom;
                ContainingBlock::new(content_width).with_height(content_height)
            },
        };

        let top_left = position
            + Vec2D {
                x: margin_left,
                y: margin_top,
            };

        let mut content_area_including_overflow = Rectangle {
            top_left,
            bottom_right: top_left,
        };

        // FIXME: Don't use the default font size here, it should be the font size
        // of this element instead
        let font_size = DEFAULT_FONT_SIZE;

        let length_resolution_context = length::ResolutionContext {
            font_size,
            root_font_size: length_resolution_context.root_font_size,
            viewport: length_resolution_context.viewport,
        };

        let content_height = match &self.contents {
            BlockContainer::BlockLevelBoxes(block_level_boxes) => {
                let mut cursor = top_left;

                cursor.y += padding_top;
                for block_box in block_level_boxes {
                    let box_fragment =
                        block_box.fragment(cursor, containing_block, length_resolution_context);
                    content_area_including_overflow
                        .grow_to_contain(box_fragment.content_area_including_overflow());
                    cursor.y += box_fragment.outer_area().height();

                    children.push(Fragment::Box(box_fragment));
                }
                cursor.y += padding_bottom;

                cursor.y - top_left.y
            },
            BlockContainer::InlineFormattingContext(inline_formatting_context) => {
                let content_top_left = top_left
                    + Vec2D {
                        x: padding_left,
                        y: padding_top,
                    };

                let (fragments, content_height) = inline_formatting_context.layout(
                    content_top_left,
                    containing_block,
                    length_resolution_context,
                );
                for fragment in &fragments {
                    content_area_including_overflow
                        .grow_to_contain(fragment.content_area_including_overflow());
                }

                children.extend_from_slice(&fragments);
                padding_top + content_height + padding_bottom
            },
        };

        // After having looked at all the children we can now actually determine the box height
        // if it wasn't defined previously
        let height = height.unwrap_or(content_height);

        let bottom_right = top_left
            + Vec2D {
                x: width,
                y: height,
            };

        let content_area = Rectangle {
            top_left,
            bottom_right,
        };

        let margin = Sides {
            top: margin_top,
            right: margin_right,
            bottom: margin_bottom,
            left: margin_left,
        };

        BoxFragment::new(
            self.node.clone(),
            self.style().clone(),
            margin,
            content_area,
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
