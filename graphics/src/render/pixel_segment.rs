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
use crate::{
    consts::TILE_SIZE,
    render::line_segment::{LineSegment, LinearRelation},
};

fn get_bounds_for_nth_segment(x: LinearRelation, y: LinearRelation, i: u32) -> f32 {
    // Approximate the ratio of x-intersections and y-intersections.
    // For example, if the x relation has a slope 5 times that of the y relation, then
    // for every 5 x-intersections there will be 1 y-intersection
    let ratio_of_x_intersections = x.slope * (x.slope + y.slope).recip();
    let ratio_of_y_intersections = y.slope * (x.slope + y.slope).recip();
    let approximate_number_of_y_intersections = (i - 1) as f32 * ratio_of_y_intersections;
    let approximate_number_of_x_intersections = (i - 1) as f32 * ratio_of_x_intersections;
    let t_of_next_x_intersection = x.solve_for(approximate_number_of_x_intersections + 1.);
    let t_of_next_y_intersection = y.solve_for(approximate_number_of_y_intersections + 1.);
    t_of_next_x_intersection.min(t_of_next_y_intersection)
}

/// The pixel segment is a packed struct with the following fields:
/// * 16 bits: Y offset of tile (signed)
/// * 16 bits: X offset of tile (signed)
/// * 8 bits: layer order (unsigned)
/// * 8 bits: cover (signed)
/// * 4 bits: X offset within tile (unsigned)
/// * 4 bits: X offset within tile (unsigned)
///
/// This ordering is not arbitrary, it allows us to easily sort by tile y, tile x, layer (which we want for rasterization)
/// by right shifting.
#[derive(Clone, Copy)]
pub struct PixelSegment(u64);

impl PixelSegment {
    pub fn new(
        tile_x: i16,
        tile_y: i16,
        layer: u8,
        cover: i8,
        in_tile_x: u8,
        in_tile_y: u8,
    ) -> Self {
        todo!()
    }
}

pub(crate) fn rasterize_line_segments(line_segments: Vec<LineSegment>) -> Vec<PixelSegment> {
    for segment in line_segments {
        for i in 0..segment.number_of_pixels {
            let lower_bound = get_bounds_for_nth_segment(segment.x_t, segment.y_t, i);
            let upper_bound = get_bounds_for_nth_segment(segment.x_t, segment.y_t, i);

            // These values represent the coordinates where the line intersects this
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
            let p0_x = segment.x_t.evaluate_at(lower_bound).round() as i32;
            let p0_y = segment.y_t.evaluate_at(lower_bound).round() as i32;
            let p1_x = segment.x_t.evaluate_at(upper_bound).round() as i32;
            let p1_y = segment.y_t.evaluate_at(upper_bound).round() as i32;

            // Compute the tile coordinate and the index of the pixel within that tile
            let tile_x = p0_x >> TILE_SIZE;
            let tile_y = p0_y >> TILE_SIZE;
            let in_tile_x = p0_x & (TILE_SIZE as i32 - 1);
            let in_tile_y = p0_y & (TILE_SIZE as i32 - 1);
        }
    }
    todo!()
}
