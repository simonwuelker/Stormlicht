//! Implements CSS2 [Float](https://drafts.csswg.org/css2/#floats) behaviour
//!
//! Also refer to [the exact rules](https://drafts.csswg.org/css2/#float-rules) that govern float behaviour.

use math::{Rectangle, Vec2D};

use crate::{
    css::{
        computed_style::ComputedStyle,
        font_metrics::DEFAULT_FONT_SIZE,
        fragment_tree::BoxFragment,
        layout::{flow::BlockFormattingContext, ContainingBlock, Pixels, Sides, Size},
        values::{
            self,
            length::{self, Length},
            AutoOr, PercentageOr,
        },
    },
    dom::{dom_objects, DomPtr},
    TreeDebug, TreeFormatter,
};

use std::{cmp, fmt, fmt::Write};

use super::BlockContainer;

#[derive(Clone)]
pub struct FloatingBox {
    pub node: DomPtr<dom_objects::Node>,
    pub style: ComputedStyle,
    pub side: Side,
    pub contents: BlockContainer,
}

impl FloatingBox {
    #[must_use]
    pub fn new(
        node: DomPtr<dom_objects::Node>,
        style: ComputedStyle,
        side: Side,
        contents: BlockContainer,
    ) -> Self {
        Self {
            node,
            style,
            side,
            contents,
        }
    }

    pub fn layout(
        &self,
        containing_block: ContainingBlock,
        ctx: length::ResolutionContext,
        float_context: &mut FloatContext,
    ) -> BoxFragment {
        let available_width = Length::pixels(containing_block.width);
        let resolve_margin = |margin: &values::Margin| {
            margin
                .map(|p| p.resolve_against(available_width))
                .map(|l| l.absolutize(ctx))
                .unwrap_or_default()
        };

        let resolve_padding =
            |padding: &values::Padding| padding.resolve_against(available_width).absolutize(ctx);

        let margin = Sides {
            top: resolve_margin(self.style.margin_top()),
            right: resolve_margin(self.style.margin_right()),
            bottom: resolve_margin(self.style.margin_bottom()),
            left: resolve_margin(self.style.margin_left()),
        };

        let padding = Sides {
            top: resolve_padding(self.style.padding_top()),
            right: resolve_padding(self.style.padding_right()),
            bottom: resolve_padding(self.style.padding_bottom()),
            left: resolve_padding(self.style.padding_left()),
        };

        let border = self
            .style
            .used_border_widths()
            .map(|side| side.absolutize(ctx));

        let width = self
            .style
            .width()
            .map(|p| p.resolve_against(available_width))
            .map(|l| l.absolutize(ctx))
            .unwrap_or_else(|| {
                todo!("compute shrink-to-fit width");
            });

        let height =
            self.style
                .height()
                .flat_map(|percentage_or_length| match percentage_or_length {
                    PercentageOr::Percentage(percentage) => {
                        if let Some(available_height) = containing_block.height() {
                            AutoOr::NotAuto(available_height * percentage.as_fraction())
                        } else {
                            AutoOr::Auto
                        }
                    },
                    PercentageOr::NotPercentage(length) => AutoOr::NotAuto(length.absolutize(ctx)),
                });

        // Layout the floats contents to determine its size
        let mut established_formatting_context = BlockFormattingContext::new(containing_block);

        let content_info = self.contents.layout(
            containing_block,
            // FIXME: this should be this elements font size
            ctx.with_font_size(DEFAULT_FONT_SIZE),
            &mut established_formatting_context,
        );

        let total_width = margin.left
            + border.left
            + padding.left
            + width
            + padding.right
            + border.right
            + margin.right;
        let total_height = margin.top
            + border.top
            + padding.top
            + height.unwrap_or(content_info.height)
            + padding.bottom
            + border.bottom
            + margin.bottom;

        let position = float_context.find_position_and_place_float_box(
            Size {
                width: total_width,
                height: total_height,
            },
            self.side,
        );

        let content_offset = Vec2D::new(
            margin.left + border.left + padding.left,
            margin.top + border.top + padding.top,
        );

        let content_area = Rectangle::from_position_and_size(
            position + content_offset,
            width,
            height.unwrap_or(content_info.height),
        );
        let padding_area = padding.surround(content_area);
        let margin_area = margin.surround(border.surround(padding_area));

        BoxFragment::new(
            Some(self.node.clone()),
            self.style.clone(),
            margin_area,
            border,
            padding_area,
            content_area,
            content_info.fragments,
        )
    }
}

#[derive(Clone, Debug)]
pub struct FloatContext {
    /// The highest y-coordinate where floats may be placed
    ///
    /// Relative to the upper edge of the containing block
    float_ceiling: Pixels,

    containing_block: ContainingBlock,

    /// Describes how the available space is reduced by floating elements
    content_bands: Vec<ContentBand>,

    lowest_float_left: Pixels,
    lowest_float_right: Pixels,
}

impl FloatContext {
    #[must_use]
    pub fn new(containing_block: ContainingBlock) -> Self {
        // Initially, there is only one float rect which makes
        // up the entire content area of the containing block
        let content_band = ContentBand {
            height: Pixels::INFINITY,
            inset_left: None,
            inset_right: None,
        };

        Self {
            float_ceiling: Pixels::ZERO,
            containing_block,
            content_bands: vec![content_band],
            lowest_float_left: Pixels::ZERO,
            lowest_float_right: Pixels::ZERO,
        }
    }

    #[must_use]
    pub fn clear_left(&self) -> Pixels {
        self.lowest_float_left
    }

    #[must_use]
    pub fn clear_right(&self) -> Pixels {
        self.lowest_float_right
    }

    #[must_use]
    pub fn clear_both(&self) -> Pixels {
        self.lowest_float_left.max(self.lowest_float_right)
    }

    pub fn lower_float_ceiling(&mut self, new_ceiling: Pixels) {
        self.float_ceiling = new_ceiling;
    }

    /// Place a float in a given position.
    ///
    /// `y_offset` describes the offset within the height of the content band specified by `content_band_index`.
    fn place_float(&mut self, margin_area: Size<Pixels>, side: Side, placement: Placement) {
        // Split the content band in up to three new bands
        let old_content_band = self.content_bands.remove(placement.band_index);
        let (new_inset_left, new_inset_right) = match side {
            Side::Left => {
                let inset_left =
                    Some(old_content_band.inset_left.unwrap_or_default() + margin_area.width);
                (inset_left, old_content_band.inset_right)
            },
            Side::Right => {
                let inset_right =
                    Some(old_content_band.inset_right.unwrap_or_default() + margin_area.width);
                (old_content_band.inset_left, inset_right)
            },
        };

        if old_content_band.height > placement.offset_in_band + margin_area.height {
            // There will be a new content_band *below* the floating box
            let area_below = ContentBand {
                height: old_content_band.height - placement.offset_in_band - margin_area.height,
                inset_left: old_content_band.inset_left,
                inset_right: old_content_band.inset_right,
            };

            self.content_bands.insert(placement.band_index, area_below);
        }

        let reduced_area = ContentBand {
            height: margin_area.height,
            inset_left: new_inset_left,
            inset_right: new_inset_right,
        };
        self.content_bands
            .insert(placement.band_index, reduced_area);

        if placement.offset_in_band != Pixels::ZERO {
            // There will be a new content band *above* the floating box
            let area_above = ContentBand {
                height: placement.offset_in_band,
                inset_left: old_content_band.inset_left,
                inset_right: old_content_band.inset_right,
            };
            self.content_bands.insert(placement.band_index, area_above);
        }

        // Lower the float ceiling: New floats may not appear above this box
        self.lower_float_ceiling(placement.position.y);

        match side {
            Side::Left => {
                self.lowest_float_left = placement.position.y + margin_area.height;
            },
            Side::Right => {
                self.lowest_float_right = placement.position.y + margin_area.height;
            },
        }
    }

    fn find_position_for_float(&self, float_width: Pixels, side: Side) -> Placement {
        debug_assert!(!self.content_bands.is_empty());

        let mut cursor = Pixels::ZERO;
        let mut band_to_place_float_in = self.content_bands.len() - 1;
        for (index, content_band) in self.content_bands[..self.content_bands.len() - 1]
            .iter()
            .enumerate()
        {
            if cursor + content_band.height < self.float_ceiling {
                // Cannot place floats above the float ceiling
                cursor += content_band.height;
                continue;
            }

            if !content_band.box_fits(float_width, side, self.containing_block.width) {
                // The float does not fit here
                cursor += content_band.height;
                continue;
            }

            // This the first suitable place to put the float
            band_to_place_float_in = index;
            break;
        }

        // The last fragment has infinite height. The float can always be placed here
        // (but won't if theres any available space earlier)
        let chosen_band = &self.content_bands[band_to_place_float_in];
        let y_position = cmp::max(cursor, self.float_ceiling);
        let x_position = match side {
            Side::Left => chosen_band.inset_left.unwrap_or_default(),
            Side::Right => {
                self.containing_block.width()
                    - float_width
                    - chosen_band.inset_right.unwrap_or_default()
            },
        };

        let position = Vec2D::new(x_position, y_position);

        Placement {
            position,
            offset_in_band: y_position - cursor,
            band_index: band_to_place_float_in,
        }
    }

    /// Finds a suitable position and updates the float state accordingly
    ///
    /// The chosen position is relative to the containing block.
    pub fn find_position_and_place_float_box(
        &mut self,
        margin_area: Size<Pixels>,
        side: Side,
    ) -> Vec2D<Pixels> {
        let placement = self.find_position_for_float(margin_area.width, side);
        self.place_float(margin_area, side, placement);
        placement.position
    }
}

#[derive(Clone, Copy, Debug)]
struct Placement {
    position: Vec2D<Pixels>,
    band_index: usize,
    offset_in_band: Pixels,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

/// A rectangular area where content may be placed
#[derive(Clone, Debug)]
struct ContentBand {
    height: Pixels,
    inset_left: Option<Pixels>,
    inset_right: Option<Pixels>,
}

impl ContentBand {
    fn box_fits(
        &self,
        box_width: Pixels,
        place_on_side: Side,
        width_of_containing_block: Pixels,
    ) -> bool {
        // The algorithm is the same for left- and right-floating boxes.
        // So instead of duplicating code, we only implement it for left-floating boxes
        // and switch the insets on the two sides if necessary

        let (this_side, opposing_side) = match place_on_side {
            Side::Left => (self.inset_left, self.inset_right),
            Side::Right => (self.inset_right, self.inset_left),
        };

        let position = this_side.unwrap_or_default();
        let right_edge = position + box_width;

        // Rule 7:
        // > A left-floating box that has another left-floating box to its left may not have its right
        // > outer edge to the right of its containing block’s right edge.
        // > An analogous rule holds for right-floating elements.
        if this_side.is_some() && width_of_containing_block < right_edge {
            return false;
        }

        // Rule 3:
        // > The right outer edge of a left-floating box may not be to the right of the left outer edge
        // > of any right-floating box that is next to it. Analogous rules hold for right-floating elements.
        if opposing_side.is_some_and(|inset| width_of_containing_block - inset < right_edge) {
            return false;
        }

        true
    }
}

impl TreeDebug for FloatingBox {
    fn tree_fmt(&self, formatter: &mut TreeFormatter<'_, '_>) -> fmt::Result {
        formatter.indent()?;
        write!(formatter, "Block Box (floating)")?;
        writeln!(formatter, " ({:?})", self.node.underlying_type())?;

        formatter.increase_indent();
        self.contents.tree_fmt(formatter)?;
        formatter.decrease_indent();
        Ok(())
    }
}
