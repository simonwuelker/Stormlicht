use std::{fmt, fmt::Write, rc::Rc};

use crate::{
    css::{
        fragment_tree::FragmentTree,
        layout::{CSSPixels, Layout, Sides, UsedSizeAndMargins},
        stylecomputer::ComputedStyle,
        values::{AutoOr, Length},
        StyleComputer,
    },
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

    pub fn fragment(self, _viewport_size: (u16, u16)) -> FragmentTree {
        todo!()
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

impl Layout for BlockLevelBox {
    fn compute_dimensions(&self, available_width: CSSPixels) -> UsedSizeAndMargins {
        // FIXME: replaced elements

        // See https://drafts.csswg.org/css2/#blockwidth for a description of how the width is computed

        // FIXME: Consider padding and borders
        let available_length = Length::pixels(available_width);
        let width = self
            .style()
            .width()
            .map(|p| p.resolve_against(available_length))
            .as_ref()
            .map(Length::absolutize);

        let mut margin_left = self
            .style()
            .margin_left()
            .map(|p| p.resolve_against(available_length))
            .as_ref()
            .map(Length::absolutize);

        let mut margin_right = self
            .style()
            .margin_right()
            .map(|p| p.resolve_against(available_length))
            .as_ref()
            .map(Length::absolutize);

        // Margins are treated as zero if the total width exceeds the available width
        let total_width_is_more_than_available = |width| {
            let total_width =
                width + margin_left.unwrap_or_default() + margin_right.unwrap_or_default();
            total_width > available_width
        };
        if width.is_not_auto_and(total_width_is_more_than_available) {
            margin_left = margin_left.or(AutoOr::NotAuto(CSSPixels::ZERO));
            margin_right = margin_right.or(AutoOr::NotAuto(CSSPixels::ZERO));
        }

        // If there is exactly one value specified as auto, its used value follows from the equality.
        let (width, margin_left, margin_right) = match (width, margin_left, margin_right) {
            (AutoOr::Auto, _, _) => (available_width, CSSPixels::ZERO, CSSPixels::ZERO),
            (AutoOr::NotAuto(width), AutoOr::Auto, AutoOr::Auto) => {
                let margin_width = (available_width - width) / 2.;
                (width, margin_width, margin_width)
            },
            (AutoOr::NotAuto(width), AutoOr::NotAuto(margin_left), AutoOr::Auto) => {
                let margin_right = available_width - margin_left;
                (width, margin_left, margin_right)
            },
            (AutoOr::NotAuto(width), AutoOr::Auto, AutoOr::NotAuto(margin_right)) => {
                let margin_left = available_width - margin_right;
                (width, margin_left, margin_right)
            },
            (AutoOr::NotAuto(width), AutoOr::NotAuto(margin_left), AutoOr::NotAuto(_)) => {
                // The values are overconstrained
                // FIXME: If the "direction" property is "rtl", we should ignore the margin left instead
                let margin_right = available_width - margin_left;
                (width, margin_left, margin_right)
            },
        };

        // Compute the height according to https://drafts.csswg.org/css2/#normal-block
        let margin_top = self
            .style()
            .margin_top()
            .map(|p| p.resolve_against(available_length))
            .as_ref()
            .map(Length::absolutize)
            .unwrap_or_default();

        let margin_bottom = self
            .style()
            .margin_bottom()
            .map(|p| p.resolve_against(available_length))
            .as_ref()
            .map(Length::absolutize)
            .unwrap_or_default();

        // FIXME
        let height = CSSPixels::ZERO;

        UsedSizeAndMargins {
            width,
            height,
            margin: Sides {
                top: margin_top,
                right: margin_right,
                bottom: margin_bottom,
                left: margin_left,
            },
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
