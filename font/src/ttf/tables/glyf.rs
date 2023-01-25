use crate::ttf::{read_i16_at, read_u16_at};
use std::fmt;

use super::loca::LocaTable;

pub struct GlyphOutlineTable<'a> {
    data: &'a [u8],
    loca_table: LocaTable<'a>,
}

impl<'a> GlyphOutlineTable<'a> {
    pub fn new(data: &'a [u8], offset: usize, length: usize, loca_table: LocaTable<'a>) -> Self {
        Self {
            data: &data[offset..][..length],
            loca_table: loca_table,
        }
    }

    pub fn get_glyph(&self, glyph_index: u16) -> Glyph<'a> {
        let offset = self.loca_table.get_glyph_offset(glyph_index);
        let data = &self.data[offset as usize..];
        Glyph(data)
    }
}

pub struct Glyph<'a>(&'a [u8]);

impl<'a> Glyph<'a> {
    pub fn is_simple(&self) -> bool {
        self.num_contours() >= 0
    }

    pub fn num_contours(&self) -> i16 {
        read_i16_at(self.0, 0)
    }

    pub fn min_x(&self) -> i16 {
        read_i16_at(self.0, 2)
    }

    pub fn min_y(&self) -> i16 {
        read_i16_at(self.0, 4)
    }

    pub fn max_x(&self) -> i16 {
        read_i16_at(self.0, 6)
    }

    pub fn max_y(&self) -> i16 {
        read_i16_at(self.0, 8)
    }

    pub fn bounding_box(&self) -> (i16, i16, i16, i16) {
        (self.min_x(), self.min_y(), self.max_x(), self.max_y())
    }

    pub fn outline(&self) -> GlyphOutline<'a> {
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

        assert!(self.is_simple());

        let num_contours = self.num_contours() as usize;
        let instruction_length = read_u16_at(self.0, 10 + num_contours * 2) as usize;

        // last value in end_points_of_contours
        let n_points = read_u16_at(self.0, 10 + num_contours * 2 - 2) + 1;

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
            let flag = GlyphFlag(self.0[first_flag_addr + num_flag_bytes_read]);
            num_flag_bytes_read += 1;

            if flag.repeat() {
                // read another byte, this is the number of times the flag should be
                // repeated
                let repeat_for = self.0[first_flag_addr + num_flag_bytes_read];
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
            data: &self.0[..total_size],
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

    pub fn instruction_length(&self) -> usize {
        read_u16_at(self.data, 10 + self.num_contours() * 2) as usize
    }

    pub fn num_points(&self) -> usize {
        // last value in end_points_of_contours
        read_u16_at(self.data, 10 + self.num_contours() * 2 - 2) as usize + 1
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
}

// TODO: this panics if the glyf is not simple :^(
impl<'a> fmt::Debug for GlyphOutline<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Glyph Outline")
            .field("is_simple", &self.is_simple())
            .field("num_contours", &self.num_contours())
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

#[derive(Clone, Copy, Debug)]
pub struct GlyphPoint {
    pub is_on_curve: bool,
    pub is_last_point_of_contour: bool,
    pub coordinates: (i16, i16),
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
    previous_point: (i16, i16),
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
            previous_point: (0, 0),
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
            GlyphCoordinateType::UnsignedDelta16B => read_u16_at(self.data, self.x_index) as i16,
            GlyphCoordinateType::ZeroDelta16B => 0,
            GlyphCoordinateType::Negative8B => -1 * self.data[self.x_index] as i16,
            GlyphCoordinateType::Positive8B => self.data[self.x_index] as i16,
        };
        self.x_index += self.current_flag.coordinate_type_x().size();

        // Update y coordinate
        let delta_y = match self.current_flag.coordinate_type_y() {
            GlyphCoordinateType::UnsignedDelta16B => read_u16_at(self.data, self.y_index) as i16,
            GlyphCoordinateType::ZeroDelta16B => 0,
            GlyphCoordinateType::Negative8B => -1 * self.data[self.y_index] as i16,
            GlyphCoordinateType::Positive8B => self.data[self.y_index] as i16,
        };
        self.y_index += self.current_flag.coordinate_type_y().size();

        let new_point = (
            self.previous_point.0 + delta_x,
            self.previous_point.1 + delta_y,
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

#[derive(Clone, Copy, Debug)]
pub struct CompoundGlyphFlag(u16);

impl CompoundGlyphFlag {
    const ARG_1_AND_2_ARE_WORDS: u16 = 1 << 0;
    const ARGS_ARE_XY_VALUES: u16 = 1 << 1;
    const ROUND_XY_TO_GRID: u16 = 1 << 2;
    const WE_HAVE_A_SCALE: u16 = 1 << 3;
    const MORE_COMPONENTS: u16 = 1 << 5;
    const WE_HAVE_AN_X_AND_Y_SCALE: u16 = 1 << 6;
    const WE_HAVE_A_TWO_BY_TWO: u16 = 1 << 7;
    const WE_HAVE_INSTRUCTIONS: u16 = 1 << 8;
    const USE_MY_METRICS: u16 = 1 << 9;
    const OVERLAP_COMPOUND: u16 = 1 << 10;

    pub fn arg_1_and_2_are_words(&self) -> bool {
        self.0 & Self::ARG_1_AND_2_ARE_WORDS != 0
    }

    pub fn args_are_xy_values(&self) -> bool {
        self.0 & Self::ARGS_ARE_XY_VALUES != 0
    }

    pub fn round_xy_to_grid(&self) -> bool {
        self.0 & Self::ROUND_XY_TO_GRID != 0
    }

    pub fn has_scale(&self) -> bool {
        self.0 & Self::WE_HAVE_A_SCALE != 0
    }

    pub fn is_last_component(&self) -> bool {
        self.0 & Self::MORE_COMPONENTS == 0
    }

    pub fn has_xy_scale(&self) -> bool {
        self.0 & Self::WE_HAVE_AN_X_AND_Y_SCALE != 0
    }

    pub fn has_two_by_two(&self) -> bool {
        self.0 & Self::WE_HAVE_A_TWO_BY_TWO != 0
    }

    pub fn has_instructions(&self) -> bool {
        self.0 & Self::WE_HAVE_INSTRUCTIONS == 0
    }

    pub fn use_my_metrics(&self) -> bool {
        self.0 & Self::USE_MY_METRICS != 0
    }

    pub fn overlap_compound(&self) -> bool {
        self.0 & Self::OVERLAP_COMPOUND != 0
    }
}
