use crate::{
    target::{BoundingBox, Point, RasterizerTarget},
    ttf::{read_i16_at, read_u16_at},
};
use std::fmt;

pub struct GlyphOutlineTable<'a>(&'a [u8]);

impl<'a> GlyphOutlineTable<'a> {
    pub fn new(data: &'a [u8], offset: usize, length: usize) -> Self {
        Self(&data[offset..][..length])
    }

    pub fn get_glyph_outline(&self, glyph_index: u32) -> GlyphOutline<'a> {
        let data = &self.0[glyph_index as usize..];
        let num_contours_maybe_negative = read_i16_at(data, 0);
        assert!(
            num_contours_maybe_negative >= 0,
            "compound glyphs are not supported"
        );
        let num_contours = num_contours_maybe_negative as usize;

        // 8 bytes of coordinates follow
        let last_index = read_u16_at(data, 10 + num_contours * 2 - 2) as usize;
        let instruction_length = read_u16_at(data, 10 + num_contours * 2) as usize;

        // Memory map is like this:
        // num contours          : i16
        // min x                 : i16
        // min y                 : i16
        // max x                 : i16
        // max y                 : i16
        // end points of contours: [u16; num contours]
        // instruction length    : u16
        // instructions          : [u8; instruction length]
        // flags                 : [u8; unknown]
        // x coords              : [u16; last value in "end points of contours" + 1]
        // y coords              : [u16; last value in "end points of contours" + 1]

        // last value in end_points_of_contours
        let n_points = read_u16_at(data, 10 + num_contours * 2 - 2) + 1;

        let first_flag_addr = 10 + num_contours * 2 + 2 + instruction_length;

        // The size of the flag array is unknown.
        // That is because a flag can repeat itself n times -
        // so we can't tell how large the flag array will be without actually iterating over it
        // once.
        // (Knowing how large the flag array is is necessary to know where the x and y coordinates
        // start)
        // Note that the flag array can never be larger than n_points (thats the case when no
        // compression happens)
        let mut remaining_flags = n_points;
        let mut num_flag_bytes_read = 0;
        let mut x_size = 0;
        let mut y_size = 0;

        while remaining_flags > 0 {
            remaining_flags -= 1;
            let flag = GlyphFlag(data[first_flag_addr + num_flag_bytes_read]);
            num_flag_bytes_read += 1;

            if flag.repeat() {
                // read another byte, this is the number of times the flag should be
                // repeated
                let repeat_for = data[first_flag_addr + num_flag_bytes_read];
                num_flag_bytes_read += 1;

                remaining_flags -= repeat_for as u16;
                x_size += flag.coordinate_type_x().size() * repeat_for as usize;
                y_size += flag.coordinate_type_y().size() * repeat_for as usize;
            }

            x_size += flag.coordinate_type_x().size();
            y_size += flag.coordinate_type_y().size();
        }
        let flag_array_size = num_flag_bytes_read;

        let total_size =
            10 + num_contours * 2 + 2 + instruction_length + flag_array_size + x_size + y_size;

        GlyphOutline {
            data: &data[..total_size],
            x_starts_at: flag_array_size,
            y_starts_at: flag_array_size + x_size,
        }
    }
}

/// Constructed via the [GlyphOutlineTable].
pub struct GlyphOutline<'a> {
    data: &'a [u8],
    // This is nontrivial to compute and we need to compute it once to create
    // the Outline, so we store it for later use instead of recomputing it
    // every time
    x_starts_at: usize,
    y_starts_at: usize,
}

impl<'a> GlyphOutline<'a> {
    pub fn is_simple(&self) -> bool {
        read_i16_at(self.data, 0) >= 0
    }

    /// If the Glyph is a simple glyph, this is the number of contours.
    /// Don't call this if the glyph is not simple.
    pub fn num_contours(&self) -> usize {
        assert!(self.is_simple());
        read_i16_at(self.data, 0) as usize
    }

    pub fn min_x(&self) -> i16 {
        read_i16_at(self.data, 2)
    }

    pub fn min_y(&self) -> i16 {
        read_i16_at(self.data, 4)
    }

    pub fn max_x(&self) -> i16 {
        read_i16_at(self.data, 6)
    }

    pub fn max_y(&self) -> i16 {
        read_i16_at(self.data, 8)
    }

    pub fn instruction_length(&self) -> usize {
        read_u16_at(self.data, 10 + self.num_contours() * 2) as usize
    }

    pub fn num_points(&self) -> usize {
        // last value in end_points_of_contours
        read_u16_at(&self.data, 10 + self.num_contours() * 2 - 2) as usize + 1
    }

    pub fn bounding_box(&self) -> BoundingBox {
        BoundingBox::new(self.min_x(), self.min_y(), self.max_x(), self.max_y())
    }

    pub fn points(&'a self) -> GlyphPointIterator<'a> {
        // The iterator only needs the data from the flags onwards
        let first_flag_addr = 10 + self.num_contours() * 2 + 2 + self.instruction_length();
        let contour_end_points = &self.data[10..][..self.num_contours() * 2];

        GlyphPointIterator::new(
            contour_end_points,
            &self.data[first_flag_addr..],
            self.num_points(),
            self.x_starts_at,
            self.y_starts_at,
        )
    }

    pub fn rasterize<T: RasterizerTarget>(&self, target: &mut T, into: BoundingBox) {
        let mut previous_point = None;
        let mut first_point_of_contour = None;
        let glyph_bb = BoundingBox::new(self.min_x(), self.min_y(), self.max_x(), self.max_y());

        for glyph_vertex in self.points() {
            let current_point_unflipped = glyph_bb.translate(glyph_vertex.coordinates, into);
            let current_point = Point::new(
                current_point_unflipped.x,
                into.height() - current_point_unflipped.y + into.min_y,
            );

            match previous_point {
                Some(previous_point) => target.line(previous_point, current_point),
                None => first_point_of_contour = Some(current_point),
            }

            if glyph_vertex.is_last_point_of_contour {
                target.line(current_point, first_point_of_contour.unwrap());
                previous_point = None;
            } else {
                previous_point = Some(current_point);
            }
        }
    }
}

// TODO: this panics if the glyf is not simple :^(
impl<'a> fmt::Debug for GlyphOutline<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Glyph Outline")
            .field("is_simple", &self.is_simple())
            .field("num_contours", &self.num_contours())
            .field("min_x", &self.min_x())
            .field("min_y", &self.min_y())
            .field("max_x", &self.max_x())
            .field("max_y", &self.max_y())
            .field("instruction_length", &self.instruction_length())
            .finish()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct GlyphFlag(u8);

impl GlyphFlag {
    const POINT_ON_CURVE: u8 = 1;
    const PRIMARY_FLAG_X: u8 = 2;
    const PRIMARY_FLAG_Y: u8 = 4;
    const REPEAT: u8 = 8;
    const SECONDARY_FLAG_X: u8 = 16;
    const SECONDARY_FLAG_Y: u8 = 32;

    pub fn is_on_curve(&self) -> bool {
        self.0 & Self::POINT_ON_CURVE != 0
    }

    pub fn repeat(&self) -> bool {
        self.0 & Self::REPEAT != 0
    }

    pub fn coordinate_type_x(&self) -> GlyphCoordinateType {
        let primary_flag = self.0 & Self::PRIMARY_FLAG_X != 0;
        let secondary_flag = self.0 & Self::SECONDARY_FLAG_X != 0;

        match (primary_flag, secondary_flag) {
            (false, false) => GlyphCoordinateType::UnsignedDelta16B,
            (false, true) => GlyphCoordinateType::ZeroDelta16B,
            (true, false) => GlyphCoordinateType::Negative8B,
            (true, true) => GlyphCoordinateType::Positive8B,
        }
    }

    pub fn coordinate_type_y(&self) -> GlyphCoordinateType {
        let primary_flag = self.0 & Self::PRIMARY_FLAG_Y != 0;
        let secondary_flag = self.0 & Self::SECONDARY_FLAG_Y != 0;

        match (primary_flag, secondary_flag) {
            (false, false) => GlyphCoordinateType::UnsignedDelta16B,
            (false, true) => GlyphCoordinateType::ZeroDelta16B,
            (true, false) => GlyphCoordinateType::Negative8B,
            (true, true) => GlyphCoordinateType::Positive8B,
        }
    }
}

#[derive(Debug)]
pub struct GlyphPoint {
    pub is_on_curve: bool,
    pub is_last_point_of_contour: bool,
    pub coordinates: Point,
}

#[derive(Debug)]
pub enum GlyphCoordinateType {
    /// The current coordinate is 16 bit signed delta change.
    UnsignedDelta16B,
    /// The current coordinate is 16 bit, has the same value as the previous one.
    ZeroDelta16B,
    /// The current coordinate is 8 bit, value is negative.
    Negative8B,
    /// The current coordinate is 8 bit, value is positive.
    Positive8B,
}

impl GlyphCoordinateType {
    pub fn size(&self) -> usize {
        match self {
            Self::ZeroDelta16B => 0,
            Self::Positive8B | Self::Negative8B => 1,
            Self::UnsignedDelta16B => 2,
        }
    }
}

pub struct GlyphPointIterator<'a> {
    contour_end_points: &'a [u8],
    data: &'a [u8],
    // TODO: make this an option (more idiomatic)
    // This requires error handling in the iterator
    current_flag: GlyphFlag,
    times_to_repeat_flag: u8,
    flag_index: usize,
    x_index: usize,
    y_index: usize,
    previous_point: Point,
    points_emitted: usize,
    contours_emitted: usize,
    num_points: usize,
}

impl<'a> GlyphPointIterator<'a> {
    pub fn new(
        contour_end_points: &'a [u8],
        data: &'a [u8],
        num_points: usize,
        x_starts_at: usize,
        y_starts_at: usize,
    ) -> Self {
        Self {
            contour_end_points: contour_end_points,
            data: data,
            current_flag: GlyphFlag(0),
            times_to_repeat_flag: 0,
            flag_index: 0,
            x_index: x_starts_at,
            y_index: y_starts_at,
            previous_point: Point::new(0, 0),
            points_emitted: 0,
            contours_emitted: 0,
            num_points: num_points,
        }
    }
}

impl<'a> Iterator for GlyphPointIterator<'a> {
    type Item = GlyphPoint;

    fn next(&mut self) -> Option<Self::Item> {
        if self.points_emitted == self.num_points {
            return None;
        }
        if self.times_to_repeat_flag == 0 {
            // Read the next flag as usual
            self.current_flag = GlyphFlag(self.data[self.flag_index]);
            self.flag_index += 1;

            if self.current_flag.repeat() {
                self.times_to_repeat_flag = self.data[self.flag_index];
                self.flag_index += 1;
            }
        } else {
            // repeat the last flag
            self.times_to_repeat_flag -= 1;
        }

        // Update x coordinate
        let delta_x = match self.current_flag.coordinate_type_x() {
            GlyphCoordinateType::UnsignedDelta16B => read_u16_at(&self.data, self.x_index) as i16,
            GlyphCoordinateType::ZeroDelta16B => 0,
            GlyphCoordinateType::Negative8B => -1 * self.data[self.x_index] as i16,
            GlyphCoordinateType::Positive8B => self.data[self.x_index] as i16,
        };
        self.x_index += self.current_flag.coordinate_type_x().size();

        // Update y coordinate
        let delta_y = match self.current_flag.coordinate_type_y() {
            GlyphCoordinateType::UnsignedDelta16B => read_u16_at(&self.data, self.y_index) as i16,
            GlyphCoordinateType::ZeroDelta16B => 0,
            GlyphCoordinateType::Negative8B => -1 * self.data[self.y_index] as i16,
            GlyphCoordinateType::Positive8B => self.data[self.y_index] as i16,
        };
        self.y_index += self.current_flag.coordinate_type_y().size();

        let new_point = Point::new(
            self.previous_point.x + delta_x,
            self.previous_point.y + delta_y,
        );
        self.previous_point = new_point;

        let is_last_point = read_u16_at(self.contour_end_points, self.contours_emitted * 2)
            as usize
            == self.points_emitted;
        if is_last_point {
            self.contours_emitted += 1;
        }

        let glyph_point = GlyphPoint {
            is_on_curve: self.current_flag.is_on_curve(), // All other flags are just relevant for parsing
            is_last_point_of_contour: is_last_point,
            coordinates: new_point,
        };
        self.points_emitted += 1;
        Some(glyph_point)
    }
}
