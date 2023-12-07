//! Implements CSS2 [Float](https://drafts.csswg.org/css2/#floats) behaviour
//!
//! Also refer to [the exact rules](https://drafts.csswg.org/css2/#float-rules) that govern float behaviour.

use math::Vec2D;

use crate::{
    css::{
        computed_style::ComputedStyle,
        layout::{ContainingBlock, Pixels, Size},
    },
    dom::{dom_objects, DomPtr},
    TreeDebug, TreeFormatter,
};

use std::{cmp, fmt, fmt::Write};

use super::BlockLevelBox;

#[derive(Clone)]
pub struct FloatingBox {
    pub node: DomPtr<dom_objects::Node>,
    pub style: ComputedStyle,
    pub side: Side,
    pub contents: Vec<BlockLevelBox>,
}

#[derive(Clone, Debug)]
pub struct FloatContext {
    /// The highest y-coordinate where floats may be placed
    ///
    /// Relative to the upper edge of the containing block
    float_ceiling: Pixels,

    containing_block: ContainingBlock,

    /// Describes how the available space is reduced by floating elements
    float_rects: Vec<FloatRect>,
}

impl FloatContext {
    #[must_use]
    pub fn new(containing_block: ContainingBlock) -> Self {
        // Initially, there is only one float rect which makes
        // up the entire content area of the containing block
        let float_rect = FloatRect {
            height: Pixels::INFINITY,
            inset_left: None,
            inset_right: None,
        };

        Self {
            float_ceiling: Pixels::ZERO,
            containing_block,
            float_rects: vec![float_rect],
        }
    }

    pub fn lower_float_ceiling(&mut self, new_ceiling: Pixels) {
        debug_assert!(
            self.float_ceiling < new_ceiling,
            "There is no reason to ever elevate the float ceiling"
        );

        self.float_ceiling = new_ceiling;
    }

    /// Place a float in a given position.
    ///
    /// `y_offset` describes the offset within the height of the content band specified by `content_band_index`.
    fn place_float(&mut self, margin_area: Size<Pixels>, side: Side, placement: Placement) {
        let old_content_band = self.float_rects.remove(placement.band_index);

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

        // Split the content band in up to three new bands
        if old_content_band.height > placement.offset_in_band + margin_area.height {
            // There will be a new content_band *below* the floating box
            let area_below = FloatRect {
                height: old_content_band.height - placement.offset_in_band - margin_area.height,
                inset_left: old_content_band.inset_left,
                inset_right: old_content_band.inset_right,
            };

            self.float_rects.insert(placement.band_index, area_below);
        }

        let reduced_area = FloatRect {
            height: margin_area.height,
            inset_left: new_inset_left,
            inset_right: new_inset_right,
        };
        self.float_rects.insert(placement.band_index, reduced_area);

        if placement.offset_in_band != Pixels::ZERO {
            // There will be a new content band *above* the floating box
            let area_above = FloatRect {
                height: placement.offset_in_band,
                inset_left: old_content_band.inset_left,
                inset_right: old_content_band.inset_right,
            };
            self.float_rects.insert(placement.band_index, area_above);
        }
    }

    fn find_position_for_float(&self, margin_area: Size<Pixels>, side: Side) -> Placement {
        debug_assert!(!self.float_rects.is_empty());

        let mut cursor = Pixels::ZERO;
        for (index, content_band) in self.float_rects[..self.float_rects.len() - 1]
            .iter()
            .enumerate()
        {
            if cursor + content_band.height < self.float_ceiling {
                // Cannot place floats above the float ceiling
                cursor += content_band.height;
                continue;
            }

            if !content_band.box_fits(margin_area, side, self.containing_block.width) {
                // The float does not fit here
                cursor += content_band.height;
                continue;
            }

            // This the first suitable place to put the float
            let y_position = cmp::max(cursor, self.float_ceiling);
            let position = Vec2D::new(content_band.inset_left.unwrap_or_default(), y_position);

            return Placement {
                position,
                offset_in_band: y_position - cursor,
                band_index: index,
            };
        }

        // The last fragment has infinite height. The float can always be placed here
        // (but won't if theres any available space earlier)
        let y_position = cmp::max(cursor, self.float_ceiling);
        let position = Vec2D::new(Pixels::ZERO, y_position);
        Placement {
            position,
            band_index: self.float_rects.len() - 1,
            offset_in_band: y_position - cursor,
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
        let placement = self.find_position_for_float(margin_area, side);
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
struct FloatRect {
    height: Pixels,
    inset_left: Option<Pixels>,
    inset_right: Option<Pixels>,
}

impl FloatRect {
    fn box_fits(
        &self,
        to_place: Size<Pixels>,
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
        let right_edge = position + to_place.width;

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
        write!(formatter, "Block Box")?;
        writeln!(formatter, " ({:?})", self.node.underlying_type())?;

        formatter.increase_indent();
        for block_box in &self.contents {
            block_box.tree_fmt(formatter)?;
        }
        formatter.decrease_indent();
        Ok(())
    }
}
