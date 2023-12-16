//! Implements functionality described in [CSS-Position Level 3](https://drafts.csswg.org/css-position)
use std::{cmp, fmt, fmt::Write};

use math::{Rectangle, Vec2D};

use crate::{
    css::{
        computed_style::ComputedStyle,
        fragment_tree::BoxFragment,
        layout::{ContainingBlock, Pixels, Sides, Size},
        values::{length, AutoOr, Inset, Length},
    },
    dom::{dom_objects, DomPtr},
    TreeDebug, TreeFormatter,
};

use super::{BlockContainer, BlockFormattingContextState};

/// A block-level box with `position: absolute;`
#[derive(Clone)]
pub struct AbsolutelyPositionedBox {
    pub node: DomPtr<dom_objects::Node>,
    pub style: ComputedStyle,
    pub content: BlockContainer,
}

/// Computes the used inset properties on a given axis
///
/// `axis_end` is assumed to be the inset of the [end edge](https://drafts.csswg.org/css-writing-modes-4/#css-end),
/// meaning that if the two insets take up more space than is available and are not `auto`,
/// then `axis_end` is assumed to be the [weaker inset](https://drafts.csswg.org/css-position/#weaker-inset) and its
/// value is changed (possibly becoming negative).
#[must_use]
fn resolve_inset_on_axis(
    axis_start: AutoOr<Pixels>,
    axis_end: AutoOr<Pixels>,
    axis_size: Pixels,
    static_position: Pixels,
) -> (Pixels, Pixels) {
    // Informally, a value of "auto" for an inset property produces an inset such that the element is positioned
    // where it was found in the flow
    match (axis_start, axis_end) {
        (AutoOr::NotAuto(l), AutoOr::NotAuto(r)) => (l, r),
        (AutoOr::NotAuto(l), AutoOr::Auto) => {
            let r = cmp::min(axis_size - l, Pixels::ZERO);
            (l, r)
        },
        (AutoOr::Auto, AutoOr::NotAuto(r)) => {
            let l = cmp::min(axis_size - r, Pixels::ZERO);
            (l, r)
        },
        (AutoOr::Auto, AutoOr::Auto) => {
            // FIXME: This is not correct, the correct results depends on the self-alignment for the axis
            // We currently always assume a self-alignment of "self-start" (or equivalent)
            (static_position, Pixels::ZERO)
        },
    }
}

/// <https://drafts.csswg.org/css-position/#inset-modified-containing-block>
///
/// The coordinates of the inset-modified containing block are relative to the actual containing block.
#[must_use]
pub fn inset_modified_containing_block(
    containing_block: Size<Pixels>,
    style: &ComputedStyle,
    length_resolution_context: length::ResolutionContext,
    static_position: Vec2D<Pixels>,
) -> Rectangle<Pixels> {
    let resolve_inset_property = |p: &Inset, resolve_percentage_against| {
        p.map(|p| p.resolve_against(resolve_percentage_against))
            .map(|l| l.absolutize(length_resolution_context))
    };

    let width = Length::pixels(containing_block.width);
    let height = Length::pixels(containing_block.height);

    let left = resolve_inset_property(style.left(), width);
    let right = resolve_inset_property(style.right(), width);
    let top = resolve_inset_property(style.top(), height);
    let bottom = resolve_inset_property(style.bottom(), height);

    let (inset_left, inset_right) =
        resolve_inset_on_axis(left, right, containing_block.width, static_position.x);
    let (inset_top, inset_bottom) =
        resolve_inset_on_axis(top, bottom, containing_block.height, static_position.y);

    let top_left = Vec2D::new(inset_left, inset_top);
    let bottom_right = Vec2D::new(containing_block.width, containing_block.height)
        - Vec2D::new(inset_right, inset_bottom);

    Rectangle::from_corners(top_left, bottom_right)
}

impl AbsolutelyPositionedBox {
    /// <https://drafts.csswg.org/css-position/#abspos-layout>
    ///
    /// `containing_block` references the nearest [absolut positioning containing block](https://drafts.csswg.org/css-position/#absolute-positioning-containing-block) and
    /// therefore always has a definite size
    pub fn layout(
        &self,
        containing_block: Size<Pixels>,
        length_resolution_context: length::ResolutionContext,
        static_position: Vec2D<Pixels>,
    ) -> BoxFragment {
        // Calculate the available space for the abspos element
        let inset_modified_block = inset_modified_containing_block(
            containing_block,
            &self.style,
            length_resolution_context,
            static_position,
        );

        // Resolve its width and height
        // https://drafts.csswg.org/css-position/#abspos-auto-size
        // FIXME: We always assume stretch-fit-size in case of "auto", but depending
        //        on the self-alignment property for the axis, it could be content-fit
        let width = self
            .style
            .width()
            .map(|p| p.resolve_against(containing_block.width.into()))
            .map(|l| l.absolutize(length_resolution_context))
            .unwrap_or(containing_block.width);

        let height = self
            .style
            .height()
            .map(|p| p.resolve_against(containing_block.height.into()))
            .map(|l| l.absolutize(length_resolution_context))
            .unwrap_or(containing_block.height);

        let content_size = Size { width, height };

        let borders = self
            .style
            .used_border_widths()
            .map(|l| l.absolutize(length_resolution_context));

        // Calculate the value of "auto" margins
        // https://drafts.csswg.org/css-position/#abspos-margins
        let margins = calculate_used_margins(
            &self.style,
            inset_modified_block,
            content_size,
            borders,
            length_resolution_context,
        );

        // Align the elements margin box within the inset modified block
        // FIXME: We don't do this yet
        let content_offset = Vec2D::new(margins.left + borders.left, margins.top + borders.top);
        let top_left = inset_modified_block.top_left() + content_offset;

        // FIXME: Actually include padding here
        let content_area = Rectangle::from_position_and_size(top_left, width, height);
        let padding_area = borders.surround(content_area);
        let margin_area = margins.surround(padding_area);

        let containing_block = ContainingBlock::new(width).with_height(height);

        // Absolute elements establish a new formatting context for their elements
        let mut formatting_context = BlockFormattingContextState::new(containing_block);

        let block_content = self.content.layout(
            containing_block,
            length_resolution_context,
            &mut formatting_context,
        );

        BoxFragment::new(
            Some(self.node.clone()),
            self.style.clone(),
            margin_area,
            borders,
            padding_area,
            content_area,
            block_content.fragments,
        )
    }
}

#[must_use]
fn calculate_used_margins(
    style: &ComputedStyle,
    inset_modified_block: Rectangle<Pixels>,
    content_size: Size<Pixels>,
    borders: Sides<Pixels>,
    ctx: length::ResolutionContext,
) -> Sides<Pixels> {
    let margin_left = style
        .margin_left()
        .map(|p| p.resolve_against(inset_modified_block.width().into()))
        .map(|l| l.absolutize(ctx));
    let margin_right = style
        .margin_right()
        .map(|p| p.resolve_against(inset_modified_block.width().into()))
        .map(|l| l.absolutize(ctx));
    let margin_top = style
        .margin_top()
        .map(|p| p.resolve_against(inset_modified_block.height().into()))
        .map(|l| l.absolutize(ctx));
    let margin_bottom = style
        .margin_bottom()
        .map(|p| p.resolve_against(inset_modified_block.height().into()))
        .map(|l| l.absolutize(ctx));

    let (margin_left, margin_right) = calculate_margins_for_axis(
        (*style.left(), *style.right()),
        inset_modified_block.width(),
        content_size.width + borders.left + borders.right, // FIXME: padding
        (margin_left, margin_right),
        true,
    );
    let (margin_top, margin_bottom) = calculate_margins_for_axis(
        (*style.top(), *style.bottom()),
        inset_modified_block.width(),
        content_size.height + borders.top + borders.bottom, // FIXME: padding
        (margin_top, margin_bottom),
        false,
    );

    Sides {
        top: margin_top,
        right: margin_right,
        bottom: margin_bottom,
        left: margin_left,
    }
}

/// Compute the used margins inside absolut layout
///
/// `axis_inset` and `margins` are expected to be passed in [flow-relative](https://drafts.csswg.org/css-writing-modes-4/#flow-relative-direction) order,
/// meaning that the first value is on the [block-start](https://drafts.csswg.org/css-writing-modes-4/#block-start)/[inline-start](https://drafts.csswg.org/css-writing-modes-4/#inline-start)
/// side and the second value is at the [block-end](https://drafts.csswg.org/css-writing-modes-4/#block-end)/[inline-end](https://drafts.csswg.org/css-writing-modes-4/#inline-end)
#[must_use]
fn calculate_margins_for_axis(
    axis_inset: (Inset, Inset),
    available_space: Pixels,
    used_box_size: Pixels,
    margins: (AutoOr<Pixels>, AutoOr<Pixels>),
    is_inline_axis: bool,
) -> (Pixels, Pixels) {
    if axis_inset.0.is_auto() || axis_inset.1.is_auto() {
        // If either inset is auto, then any auto margin resolves to zero
        (
            margins.0.unwrap_or(Pixels::ZERO),
            margins.1.unwrap_or(Pixels::ZERO),
        )
    } else {
        let remaining_space = available_space - used_box_size;

        // The remaining space is distributed among any auto margins
        match (margins.0, margins.1) {
            (AutoOr::Auto, AutoOr::Auto) => {
                if is_inline_axis && remaining_space.is_sign_negative() {
                    (Pixels::ZERO, remaining_space)
                } else {
                    (remaining_space / 2., remaining_space / 2.)
                }
            },
            (AutoOr::Auto, AutoOr::NotAuto(r)) => (remaining_space - r, r),
            (AutoOr::NotAuto(l), AutoOr::Auto) => (l, remaining_space - l),
            (AutoOr::NotAuto(l), AutoOr::NotAuto(r)) => (l, r),
        }
    }
}

impl TreeDebug for AbsolutelyPositionedBox {
    fn tree_fmt(&self, formatter: &mut TreeFormatter<'_, '_>) -> fmt::Result {
        formatter.indent()?;
        write!(formatter, "Block Box")?;
        writeln!(formatter, " ({:?})", self.node.underlying_type())?;

        formatter.increase_indent();
        self.content.tree_fmt(formatter)?;
        formatter.decrease_indent();
        Ok(())
    }
}
