use crate::line_segment::LineSegment;

pub const TILE_SIZE: usize = 16;

#[derive(Clone, Copy, Debug)]
pub struct PixelSegment {}

pub(crate) fn rasterize_line_segments(line_segments: Vec<LineSegment>) -> Vec<PixelSegment> {
    for line_segment in line_segments {
        for pixel_segment_index in 0..line_segment.pixel_segments_touched {
            todo!()
        }
    }
    todo!()
}
