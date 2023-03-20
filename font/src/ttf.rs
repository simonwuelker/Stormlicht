//! [TrueType](https://developer.apple.com/fonts/TrueType-Reference-Manual) font parser
//!
//! ## Reference Material:
//! * <https://learn.microsoft.com/en-us/typography/opentype/spec/otff>
//! * <https://formats.kaitai.io/ttf/index.html>
//! * <https://handmade.network/forums/articles/t/7330-implementing_a_font_reader_and_rasterizer_from_scratch%252C_part_1__ttf_font_reader>

use crate::{
    path::{DiscretePoint, Operation, PathReader},
    ttf_tables::{
        cmap,
        glyf::{self, Glyph},
        head, hhea,
        hmtx::{self, LongHorMetric},
        loca, maxp, name,
        offset::OffsetTable,
    },
    Point, Rasterizer,
};
use canvas::{Canvas, Drawable};
use thiserror::Error;

const DEFAULT_FONT: &[u8; 168644] =
    include_bytes!("../../downloads/fonts/roboto/Roboto-Medium.ttf");

const CMAP_TAG: u32 = u32::from_be_bytes(*b"cmap");
const HEAD_TAG: u32 = u32::from_be_bytes(*b"head");
const LOCA_TAG: u32 = u32::from_be_bytes(*b"loca");
const GLYF_TAG: u32 = u32::from_be_bytes(*b"glyf");
const HHEA_TAG: u32 = u32::from_be_bytes(*b"hhea");
const HMTX_TAG: u32 = u32::from_be_bytes(*b"hmtx");
const MAXP_TAG: u32 = u32::from_be_bytes(*b"maxp");
const NAME_TAG: u32 = u32::from_be_bytes(*b"name");
const _VHEA_TAG: u32 = u32::from_be_bytes(*b"vhea");

#[derive(Debug, Error)]
pub enum TTFParseError {
    #[error("Unexpected end of file")]
    UnexpectedEOF,
    #[error("Unsupported ttf format")]
    UnsupportedFormat,
    #[error("Missing required table")]
    MissingTable,
}

pub struct Font<'a> {
    offset_table: OffsetTable<'a>,
    head_table: head::HeadTable<'a>,
    format4: cmap::Format4<'a>,
    glyph_table: glyf::GlyphOutlineTable<'a>,
    hmtx_table: hmtx::HMTXTable<'a>,
    maxp_table: maxp::MaxPTable<'a>,
    name_table: name::NameTable<'a>,
}

impl<'a> Font<'a> {
    pub fn new(data: &'a [u8]) -> Result<Self, TTFParseError> {
        let offset_table = OffsetTable::new(data);
        if offset_table.scaler_type() != 0x00010000 {
            return Err(TTFParseError::UnsupportedFormat);
        }

        let head_entry = offset_table
            .get_table(HEAD_TAG)
            .ok_or(TTFParseError::MissingTable)?;
        let head_table = head::HeadTable::new(data, head_entry.offset());

        let cmap_entry = offset_table
            .get_table(CMAP_TAG)
            .ok_or(TTFParseError::MissingTable)?;
        let cmap_table = cmap::CMAPTable::new(data, cmap_entry.offset());

        let unicode_table_or_none = cmap_table.get_subtable_for_platform(cmap::PlatformID::Unicode);

        let unicode_table_offset = unicode_table_or_none.ok_or(TTFParseError::MissingTable)?;
        let format4 = cmap::Format4::new(data, cmap_entry.offset() + unicode_table_offset);

        let loca_entry = offset_table
            .get_table(LOCA_TAG)
            .ok_or(TTFParseError::MissingTable)?;
        let loca_table =
            loca::LocaTable::new(data, loca_entry.offset(), head_table.index_to_loc_format());

        let glyf_entry = offset_table
            .get_table(GLYF_TAG)
            .ok_or(TTFParseError::MissingTable)?;
        let glyph_table = glyf::GlyphOutlineTable::new(
            data,
            glyf_entry.offset(),
            glyf_entry.length(),
            loca_table,
        );

        let hhea_entry = offset_table
            .get_table(HHEA_TAG)
            .ok_or(TTFParseError::MissingTable)?;
        let hhea_table = hhea::HHEATable::new(data, hhea_entry.offset());

        let hmtx_entry = offset_table
            .get_table(HMTX_TAG)
            .ok_or(TTFParseError::MissingTable)?;
        let hmtx_table = hmtx::HMTXTable::new(
            data,
            hmtx_entry.offset(),
            hhea_table.num_of_long_hor_metrics(),
        );

        let maxp_entry = offset_table
            .get_table(MAXP_TAG)
            .ok_or(TTFParseError::MissingTable)?;
        let maxp_table = maxp::MaxPTable::new(data, maxp_entry.offset());

        let name_entry = offset_table
            .get_table(NAME_TAG)
            .ok_or(TTFParseError::MissingTable)?;
        let name_table = name::NameTable::new(data, name_entry.offset()).unwrap();

        Ok(Self {
            offset_table,
            head_table,
            format4,
            glyph_table,
            hmtx_table,
            maxp_table,
            name_table,
        })
    }

    /// Get the total number of glyphs defined in the font
    pub fn num_glyphs(&self) -> usize {
        self.maxp_table.num_glyphs()
    }

    // TODO: support more than one cmap format table (format 4 seems to be the most common but still)
    pub fn format_4(&self) -> &cmap::Format4<'a> {
        &self.format4
    }

    /// Get the full name of the font, if specified.
    /// Fonts will usually specify their own name, though it is not required.
    pub fn name(&self) -> Option<String> {
        self.name_table.get_font_name()
    }

    pub fn glyf(&self) -> &glyf::GlyphOutlineTable<'a> {
        &self.glyph_table
    }

    pub fn hmtx(&self) -> &hmtx::HMTXTable<'a> {
        &self.hmtx_table
    }

    pub fn head(&self) -> &head::HeadTable<'a> {
        &self.head_table
    }

    pub fn offset_table(&self) -> &OffsetTable<'a> {
        &self.offset_table
    }

    /// Get the Glyph index for a given codepoint
    pub fn get_glyph_index(&self, codepoint: u16) -> Option<u16> {
        self.format4.get_glyph_index(codepoint)
    }

    pub fn get_glyph(&self, glyph_id: u16) -> Result<Glyph<'a>, TTFParseError> {
        // Any character that does not exist is mapped to index zero, which is defined to be the
        // missing character glyph
        let glyph = self.glyph_table.get_glyph(glyph_id);
        Ok(glyph)
    }

    /// Return the number of coordinate points per font size unit.
    /// This value is used to scale fonts, ie. when you render a font with
    /// size `17px`, one `em` equals `17px`.
    ///
    /// Note that this value does not constrain the size of individual glyphs.
    /// A glyph may have a size larger than `1em`.
    pub fn units_per_em(&self) -> u16 {
        self.head_table.units_per_em()
    }

    pub fn rasterize(&self, text: &str, bitmap: &mut Canvas, position: (i16, i16), font_size: f32) {
        let (mut x, y) = position;

        for c in text.chars() {
            let glyph_id = self.get_glyph_index(c as u16).unwrap_or(0);
            let glyph = self.get_glyph(glyph_id).unwrap();

            let horizontal_metrics = self.hmtx_table.get_metric_for(glyph_id);

            let glyph_x =
                x + self.scale(horizontal_metrics.left_side_bearing() as f32, font_size) as i16;
            let glyph_y = y;
            self.rasterize_glyph(
                glyph,
                bitmap,
                (glyph_x, glyph_y),
                font_size,
                horizontal_metrics,
            );
            x += self.scale(horizontal_metrics.advance_width() as f32, font_size) as i16;
        }
    }

    /// Rasterize a single glyph at a given position on a bitmap.
    /// Note that this function does **not** care about positioning whatsoever:
    /// The given position will contain the top left edge of the smallest bounding rectangle
    /// for the glyph. That means that `left_side_bearing` and `top_side_bearing`
    /// need to be taken care of by the caller.
    fn rasterize_glyph(
        &self,
        glyph: Glyph,
        bitmap: &mut Canvas,
        position: (i16, i16),
        font_size: f32,
        horizontal_metric: LongHorMetric,
    ) {
        let left_side_bearing = self.scale(horizontal_metric.left_side_bearing(), font_size) as i16;
        match glyph {
            Glyph::Simple(simple_glyph) => {
                let top_side_bearing = self.scale(
                    self.units_per_em() as i16 - simple_glyph.metrics.max_y,
                    font_size,
                ) as i16;

                let glyph_width = self
                    .scale(simple_glyph.metrics.width() as f32, font_size)
                    .ceil() as usize
                    + 1;
                let glyph_height = self
                    .scale(simple_glyph.metrics.height() as f32, font_size)
                    .ceil() as usize
                    + 1;

                let mut rasterizer = Rasterizer::new(glyph_width, glyph_height);
                let scale_point = |glyph_point: DiscretePoint| Point {
                    x: self.scale(
                        (glyph_point.x - simple_glyph.metrics.min_x) as f32,
                        font_size,
                    ),
                    y: self.scale(
                        (glyph_point.y - simple_glyph.metrics.min_y) as f32,
                        font_size,
                    ),
                };

                // Draw the outlines of the glyph on the rasterizer buffer
                let mut write_head = Point::default();
                let path_operations = PathReader::new(simple_glyph.into_iter());
                for path_op in path_operations {
                    match path_op {
                        Operation::MoveTo(destination) => write_head = scale_point(destination),
                        Operation::LineTo(destination) => {
                            let scaled_destionation = scale_point(destination);
                            rasterizer.draw_line(write_head, scaled_destionation);
                            write_head = scaled_destionation;
                        },
                        Operation::QuadBezTo(p1, p2) => {
                            let scaled_p2 = scale_point(p2);
                            rasterizer.draw_quad_bezier(write_head, scale_point(p1), scaled_p2);
                            write_head = scaled_p2;
                        },
                    }
                }

                // Translate the rasterized glyph onto the canvas
                rasterizer.for_each_pixel(|coords, opacity| {
                    let translated_x = (position.0 + left_side_bearing) as usize + coords.0;
                    let translated_y =
                        (position.1 + top_side_bearing) as usize + glyph_height - 1 - coords.1;

                    let color = [255 - opacity; 3];

                    // If the pixel is already darker, keep it as-is
                    if 255 - opacity < bitmap.pixel_at(translated_x, translated_y)[0] {
                        bitmap
                            .pixel_at_mut(translated_x, translated_y)
                            .copy_from_slice(&color);
                    }
                });
            },
            Glyph::Compound(compound_glyph) => {
                for glyph_component in compound_glyph {
                    let glyph_x =
                        position.0 + self.scale(glyph_component.x_offset, font_size) as i16;
                    let glyph_y =
                        position.1 - self.scale(glyph_component.y_offset, font_size) as i16;

                    let referenced_glyph = self.get_glyph(glyph_component.glyph_index).unwrap();
                    self.rasterize_glyph(
                        referenced_glyph,
                        bitmap,
                        (glyph_x, glyph_y),
                        font_size,
                        LongHorMetric::default(),
                    );
                }
            },
        }
    }

    /// Converts a value from font units to pixel size
    fn scale<V: Into<f32>>(&self, value: V, font_size: f32) -> f32 {
        (value.into() * font_size) / self.units_per_em() as f32
    }
}

impl Default for Font<'static> {
    fn default() -> Self {
        Self::new(DEFAULT_FONT).unwrap()
    }
}

pub fn read_u16_at(data: &[u8], offset: usize) -> u16 {
    u16::from_be_bytes(data[offset..offset + 2].try_into().unwrap())
}

pub fn read_u32_at(data: &[u8], offset: usize) -> u32 {
    u32::from_be_bytes(data[offset..offset + 4].try_into().unwrap())
}

pub fn read_i16_at(data: &[u8], offset: usize) -> i16 {
    i16::from_be_bytes(data[offset..offset + 2].try_into().unwrap())
}
