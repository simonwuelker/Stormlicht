//! Computes the coverage for each pixel along a line segment
//!
//! <svg width="100%" height="100%" xmlns="http://www.w3.org/2000/svg">
//!  <defs>
//!  <pattern id="smallGrid" width="8" height="8" patternUnits="userSpaceOnUse">
//!   <path d="M 8 0 L 0 0 0 8" fill="none" stroke="gray" stroke-width="0.5"/>
//!  </pattern>
//!  <pattern id="grid" width="80" height="80" patternUnits="userSpaceOnUse">
//!    <rect width="80" height="80" fill="url(#smallGrid)"/>
//!    <path d="M 80 0 L 0 0 0 80" fill="none" stroke="gray" stroke-width="1"/>
//!  </pattern>
//! </defs>
//! <rect width="100%" height="100%" fill="url(#grid)" />
//! </svg>
use crate::{consts::TILE_SIZE, render::line_segment::LineSegment};

/// The pixel segment is a packed struct with the following fields:
/// * 16 bits: Y offset of tile (signed)
/// * 16 bits: X offset of tile (signed)
/// * 16 bits: layer order (unsigned)
/// * 8 bits: vertical portion of the pixel covered cover (signed, negative values indicate line direction)
/// * 4 bits: X offset within tile (unsigned)
/// * 4 bits: X offset within tile (unsigned)
///
/// This ordering is not arbitrary, it allows us to easily sort by tile y, tile x, layer (which we want for rasterization)
/// by right shifting.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PixelSegment(u64);

impl PixelSegment {
    #[inline]
    pub fn new(
        tile_x: i32,
        tile_y: i32,
        layer: u16,
        vertical_cover: i8,
        in_tile_x: i32,
        in_tile_y: i32,
    ) -> Self {
        debug_assert!(i16::MIN as i32 <= tile_x && tile_x <= i16::MAX as i32);
        debug_assert!(i16::MIN as i32 <= tile_y && tile_y <= i16::MAX as i32);
        debug_assert!((in_tile_x as usize) < TILE_SIZE);
        debug_assert!((in_tile_y as usize) < TILE_SIZE);

        let mut result = 0;

        result |= (tile_y as u16 as u64) << 48;
        result |= (tile_x as u16 as u64) << 32;
        result |= (layer as u64) << 16;
        result |= (vertical_cover as u8 as u64) << 8;
        result |= (in_tile_x as u64) << 4;
        result |= in_tile_y as u64;
        Self(result)
    }

    #[inline]
    pub fn tile_y(&self) -> i16 {
        (self.0 >> 48) as i16
    }

    #[inline]
    pub fn tile_x(&self) -> i16 {
        (self.0 >> 32) as i16
    }

    #[inline]
    pub fn layer(&self) -> u16 {
        (self.0 >> 16) as u16
    }

    #[inline]
    pub fn vertical_cover(&self) -> i8 {
        (self.0 >> 8) as i8
    }

    #[inline]
    pub fn in_tile_x(&self) -> u8 {
        ((self.0 >> 4) & 0b1111) as u8
    }

    #[inline]
    pub fn in_tile_y(&self) -> u8 {
        self.0 as u8 & 0b1111
    }
}

/// Finds the `t` value for the `i`-th pixel segment on a line segment.
#[inline]
fn get_intersection_by_index(segment: &LineSegment, index: u32) -> f32 {
    // Compute the portion of intersections that are x-intersections and y-intersecionts
    let slope_sum = segment.t_x.slope + segment.t_y.slope;

    let ratio_of_x_intersections = segment.t_x.slope / slope_sum;
    let ratio_of_y_intersections = 1. - ratio_of_x_intersections;

    // Compute the x coordinate of the next x intersection and the y coordinate of the next y intersection
    debug_assert!(
        segment.t_y.slope.is_finite(),
        "Horizontal lines should have been filtered before"
    );

    if segment.t_x.slope.is_finite() {
        let next_x_intersection = (index as f32 * ratio_of_x_intersections).ceil();
        let next_y_intersection = (index as f32 * ratio_of_y_intersections).ceil();

        // Now we have two candidates for the next intersection: It's either a horizontal intersection (next_x_intersection)
        // or a vertical intersection (next_y_intersection)
        //
        // To find out which one it is, we simply compute the `t` for both of them and choose the smaller value.
        // Note that both values might be equal (if the next intersection is exactly on a pixel corner)
        let t_of_next_x_intersection = segment.t_x.evaluate_at(next_x_intersection);
        let t_of_next_y_intersection = segment.t_y.evaluate_at(next_y_intersection);
        t_of_next_x_intersection.min(t_of_next_y_intersection)
    } else {
        // This is a vertical line. The next intersection is guaranteed to be on a horizontal boundary
        segment.t_y.evaluate_at(index as f32)
    }
}

/// Finds the bounds (`t0`, `t1`) that describe the portion of a line within a certain pixel
#[inline]
fn get_segment_bounds(segment: &LineSegment, index: u32) -> (f32, f32) {
    (
        get_intersection_by_index(segment, index).max(0.),
        get_intersection_by_index(segment, index + 1).min(1.),
    )
}

pub(crate) fn rasterize_line_segments(line_segments: Vec<LineSegment>) -> Vec<PixelSegment> {
    let number_of_pixel_segments: u32 = line_segments
        .iter()
        .map(|segment| segment.number_of_pixels)
        .sum();
    let mut pixel_segments = Vec::with_capacity(number_of_pixel_segments as usize);

    for segment in line_segments {
        for i in 0..segment.number_of_pixels {
            let (lower_bound, upper_bound) = get_segment_bounds(&segment, i);

            // These values represent the coordinates (in sub-pixel space) where the line intersects this
            // specific pixel
            //
            //       (p2)/
            //      ╔═══x═════════╗
            //      ║  /          ║
            //      ║ /           ║
            //      ║/  <pixel>   ║
            // (p1) x             ║
            //     /║             ║
            //    / ╚═════════════╝
            //
            // Note that the start & end points of the line are not necessarily on the pixel outline,
            // they may instead be in the middle of it.
            let p0_x = segment.x_t.evaluate_at(lower_bound).round() as i32;
            let p0_y = segment.y_t.evaluate_at(lower_bound).round() as i32;
            let p1_x = segment.x_t.evaluate_at(upper_bound).round() as i32;
            let p1_y = segment.y_t.evaluate_at(upper_bound).round() as i32;

            // Compute the tile coordinate and the index of the pixel within that tile
            let tile_x = p0_x >> TILE_SIZE;
            let tile_y = p0_y >> TILE_SIZE;
            let in_tile_x = p0_x & (TILE_SIZE as i32 - 1);
            let in_tile_y = p0_y & (TILE_SIZE as i32 - 1);
            pixel_segments.push(PixelSegment::new(
                tile_x,
                tile_y,
                segment.layer_index,
                (p1_y - p0_y) as i8,
                in_tile_x,
                in_tile_y,
            ))
        }
    }

    // The painter expects the pixel segments to be sorted as specified in the PartialOrd
    // implementation of PixelSegment
    pixel_segments.sort_unstable();

    pixel_segments
}

impl PartialOrd for PixelSegment {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PixelSegment {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Comparison makes use of the layout of a pixel segment in memory.
        // The following values are compared (in order of importance):
        // * Tile Y Coordinate
        // * Tile X Coordinate
        // * Segment Layer Index

        // These values make up the first 40 bits of the 64 bit segment, so we shift
        // by 24 bits to remove unnecessary fields
        (self.0 >> 24).cmp(&(other.0 >> 24))
    }
}

#[cfg(test)]
mod tests {
    use super::PixelSegment;

    #[test]
    fn pixel_segment() {
        let pixel_segment = PixelSegment::new(-5, 4, 3, -10, 1, 0);

        assert_eq!(pixel_segment.tile_x(), -5);
        assert_eq!(pixel_segment.tile_y(), 4);
        assert_eq!(pixel_segment.layer(), 3);
        assert_eq!(pixel_segment.vertical_cover(), -10);
        assert_eq!(pixel_segment.in_tile_x(), 1);
        assert_eq!(pixel_segment.in_tile_y(), 0);
    }
}
