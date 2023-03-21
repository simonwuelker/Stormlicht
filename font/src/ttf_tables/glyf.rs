//! [Glyph](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6glyf.html) table implementation

use super::loca::LocaTable;
use crate::{
    path::DiscretePoint,
    ttf::{read_i16_at, read_u16_at},
    Stream,
};
use std::fmt;

/// The maximum number of components that a glyph may reference during
/// outline calculation. Note that this number is not necessarily equal to
/// the number of components actually used, because compound glyphs are counted
/// as well even though they "ultimately" only consist of a number of other
/// glyphs.
const _MAX_COMPONENTS: usize = 10;

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
        Glyph::from_data(data)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Metrics {
    pub min_x: i16,
    pub min_y: i16,
    pub max_x: i16,
    pub max_y: i16,
}

impl Metrics {
    pub fn width(&self) -> i16 {
        self.max_x - self.min_x
    }

    pub fn height(&self) -> i16 {
        self.max_y - self.min_y
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Glyph<'a> {
    Simple(SimpleGlyph<'a>),
    Compound(CompoundGlyph<'a>),
}

#[derive(Clone, Copy, Debug)]
pub struct SimpleGlyph<'a> {
    pub metrics: Metrics,
    pub num_points: usize,
    pub end_points_of_contours: &'a [u8],
    pub instructions: &'a [u8],
    pub flags: &'a [u8],
    pub x_coordinates: &'a [u8],
    pub y_coordinates: &'a [u8],
}

#[derive(Clone, Copy, Debug)]
pub struct CompoundGlyph<'a> {
    data: Stream<'a>,
    done: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct CompoundGlyphComponent {
    pub component_flag: CompoundGlyphFlag,
    pub glyph_index: u16,
    pub x_offset: i16,
    pub y_offset: i16,
}

impl<'a> Glyph<'a> {
    pub fn from_data(data: &'a [u8]) -> Self {
        // Memory map is like this (same for simple & compound glyphs):
        // num contours          : i16
        // min x                 : i16
        // min y                 : i16
        // max x                 : i16
        // max y                 : i16
        let metrics = Metrics {
            min_x: read_i16_at(data, 2),
            min_y: read_i16_at(data, 4),
            max_x: read_i16_at(data, 6),
            max_y: read_i16_at(data, 8),
        };

        let num_contours = read_i16_at(data, 0);

        // A glyph is simple if num_contours is positive or zero
        if !num_contours.is_negative() {
            // Simple glyphs are structured as follows:
            //
            // end points of contours: [u16; num contours]
            // instruction length    : u16
            // instructions          : [u8; instruction length]
            // flags                 : [u8; unknown]
            // x coords              : [u16; last value in "end points of contours" + 1]
            // y coords              : [u16; last value in "end points of contours" + 1]

            let num_contours = num_contours as usize;
            let end_points_of_contours = &data[10..][..num_contours * 2];

            let instruction_length = read_u16_at(data, 10 + num_contours * 2) as usize;
            let instructions = &data[10 + num_contours * 2 + 2..][..instruction_length];

            // last value in end_points_of_contours
            let num_points = read_u16_at(data, 10 + num_contours * 2 - 2) as usize + 1;

            let first_flag_addr = 10 + num_contours * 2 + 2 + instruction_length;

            // The size of the flag array is unknown.
            // That is because a flag can repeat itself n times -
            // so we can't tell how large the flag array will be without actually iterating over it
            // once.
            // (Knowing how large the flag array is is necessary to know where the x and y coordinates
            // start)
            // Note that the flag array can never be larger than n_points (thats the case when no
            // compression happens)
            let mut remaining_flags = num_points;
            let mut num_flag_bytes_read = 0;
            let mut x_size = 0;
            let mut y_size = 0;

            // Read flags until we have one flag for each point
            while remaining_flags > 0 {
                remaining_flags -= 1;
                let flag = GlyphFlag(data[first_flag_addr + num_flag_bytes_read]);
                num_flag_bytes_read += 1;

                if flag.repeat() {
                    // read another byte, this is the number of times the flag should be
                    // repeated
                    let repeat_for = data[first_flag_addr + num_flag_bytes_read];
                    num_flag_bytes_read += 1;

                    remaining_flags -= repeat_for as usize;
                    x_size += flag.coordinate_type_x().size() * repeat_for as usize;
                    y_size += flag.coordinate_type_y().size() * repeat_for as usize;
                }

                x_size += flag.coordinate_type_x().size();
                y_size += flag.coordinate_type_y().size();
            }
            let flags = &data[first_flag_addr..][..num_flag_bytes_read];
            let x_coordinates = &data[first_flag_addr + num_flag_bytes_read..][..x_size];
            let y_coordinates = &data[first_flag_addr + num_flag_bytes_read + x_size..][..y_size];

            Self::Simple(SimpleGlyph {
                metrics,
                num_points,
                end_points_of_contours,
                instructions,
                flags,
                x_coordinates,
                y_coordinates,
            })
        } else {
            // Memory map for compound glyphs looks like this:
            //
            // component flag: u16                       \
            // glyph index: u16                           |
            // X offset, type depends on component flags  | Repeated any number
            // Y offset, type depends on component flags  | of times
            // Transformation options                    /
            Self::Compound(CompoundGlyph {
                data: Stream::new(&data[10..]),
                done: false,
            })
        }
    }
}

impl<'a> Iterator for CompoundGlyph<'a> {
    type Item = CompoundGlyphComponent;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        let component_flag = CompoundGlyphFlag(self.data.read::<u16>().unwrap());
        let referenced_glyph_index = self.data.read::<u16>().unwrap();

        if component_flag.is_last_component() {
            self.done = true;
        }

        let (x_offset, y_offset) = match (
            component_flag.arg_1_and_2_are_words(),
            component_flag.args_are_xy_values(),
        ) {
            (false, false) => {
                // u8
                (
                    self.data.read::<u8>().unwrap() as i16,
                    self.data.read::<u8>().unwrap() as i16,
                )
            },
            (false, true) => {
                // i8
                (
                    self.data.read::<i8>().unwrap() as i16,
                    self.data.read::<i8>().unwrap() as i16,
                )
            },
            (true, false) => {
                // u16
                (
                    self.data.read::<u16>().unwrap() as i16,
                    self.data.read::<u16>().unwrap() as i16,
                )
            },
            (true, true) => {
                // i16
                (
                    self.data.read::<i16>().unwrap(),
                    self.data.read::<i16>().unwrap(),
                )
            },
        };

        // TODO: Transformations

        Some(CompoundGlyphComponent {
            component_flag: component_flag,
            glyph_index: referenced_glyph_index,
            x_offset,
            y_offset,
        })
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
    pub coordinates: DiscretePoint,
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
    flags: &'a [u8],
    x_coordinates: &'a [u8],
    y_coordinates: &'a [u8],
    // TODO: make this an option (more idiomatic)
    // This requires error handling in the iterator
    current_flag: GlyphFlag,
    times_to_repeat_flag: u8,
    flag_index: usize,
    x_index: usize,
    y_index: usize,
    previous_point: DiscretePoint,
    points_emitted: usize,
    contours_emitted: usize,
    num_points: usize,
}

impl<'a> GlyphPointIterator<'a> {
    pub fn new(
        contour_end_points: &'a [u8],
        flags: &'a [u8],
        x_coordinates: &'a [u8],
        y_coordinates: &'a [u8],
        num_points: usize,
    ) -> Self {
        Self {
            contour_end_points,
            flags,
            x_coordinates,
            y_coordinates,
            current_flag: GlyphFlag(0),
            times_to_repeat_flag: 0,
            flag_index: 0,
            x_index: 0,
            y_index: 0,
            previous_point: DiscretePoint::origin(),
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
            self.current_flag = GlyphFlag(self.flags[self.flag_index]);
            self.flag_index += 1;

            if self.current_flag.repeat() {
                self.times_to_repeat_flag = self.flags[self.flag_index];
                self.flag_index += 1;
            }
        } else {
            // repeat the last flag
            self.times_to_repeat_flag -= 1;
        }

        // Update x coordinate
        let delta_x = match self.current_flag.coordinate_type_x() {
            GlyphCoordinateType::UnsignedDelta16B => {
                read_u16_at(self.x_coordinates, self.x_index) as i16
            },
            GlyphCoordinateType::ZeroDelta16B => 0,
            GlyphCoordinateType::Negative8B => -1 * self.x_coordinates[self.x_index] as i16,
            GlyphCoordinateType::Positive8B => self.x_coordinates[self.x_index] as i16,
        };
        self.x_index += self.current_flag.coordinate_type_x().size();

        // Update y coordinate
        let delta_y = match self.current_flag.coordinate_type_y() {
            GlyphCoordinateType::UnsignedDelta16B => {
                read_u16_at(self.y_coordinates, self.y_index) as i16
            },
            GlyphCoordinateType::ZeroDelta16B => 0,
            GlyphCoordinateType::Negative8B => -1 * self.y_coordinates[self.y_index] as i16,
            GlyphCoordinateType::Positive8B => self.y_coordinates[self.y_index] as i16,
        };
        self.y_index += self.current_flag.coordinate_type_y().size();

        let new_point = DiscretePoint {
            x: self.previous_point.x + delta_x,
            y: self.previous_point.y + delta_y,
        };
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

#[derive(Clone, Copy)]
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

impl fmt::Debug for CompoundGlyphFlag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Compound Glyph Flag")
            .field("arg 1 and 2 are words", &self.arg_1_and_2_are_words())
            .field("args are xy values", &self.args_are_xy_values())
            .field("round xy to grid", &self.round_xy_to_grid())
            .field("has scale", &self.has_scale())
            .field("is last component", &self.is_last_component())
            .field("has xy scale", &self.has_xy_scale())
            .field("has two by two", &self.has_two_by_two())
            .field("has instructions", &self.has_instructions())
            .field("use my metrics", &self.use_my_metrics())
            .field("overlap compound", &self.overlap_compound())
            .finish()
    }
}

impl<'a> IntoIterator for SimpleGlyph<'a> {
    type IntoIter = GlyphPointIterator<'a>;
    type Item = GlyphPoint;

    fn into_iter(self) -> Self::IntoIter {
        GlyphPointIterator::new(
            self.end_points_of_contours,
            self.flags,
            self.x_coordinates,
            self.y_coordinates,
            self.num_points,
        )
    }
}
