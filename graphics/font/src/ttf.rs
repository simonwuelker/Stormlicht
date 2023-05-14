//! [TrueType](https://developer.apple.com/fonts/TrueType-Reference-Manual) font parser
//!
//! ## Reference Material:
//! * <https://learn.microsoft.com/en-us/typography/opentype/spec/otff>
//! * <https://formats.kaitai.io/ttf/index.html>
//! * <https://handmade.network/forums/articles/t/7330-implementing_a_font_reader_and_rasterizer_from_scratch%252C_part_1__ttf_font_reader>

use crate::{
    path::{DiscretePoint, Operation, PathReader},
    ttf_tables::{
        cmap::{self, GlyphID},
        glyf::{self, CompoundGlyph, Glyph, GlyphPointIterator, Metrics},
        head, hhea, hmtx, loca, maxp, name,
        offset::OffsetTable,
    },
    Point, Rasterizer,
};
use canvas::{Canvas, Drawable};

const DEFAULT_FONT: &[u8; 168644] =
    include_bytes!("../../../downloads/fonts/roboto/Roboto-Medium.ttf");

const CMAP_TAG: u32 = u32::from_be_bytes(*b"cmap");
const HEAD_TAG: u32 = u32::from_be_bytes(*b"head");
const LOCA_TAG: u32 = u32::from_be_bytes(*b"loca");
const GLYF_TAG: u32 = u32::from_be_bytes(*b"glyf");
const HHEA_TAG: u32 = u32::from_be_bytes(*b"hhea");
const HMTX_TAG: u32 = u32::from_be_bytes(*b"hmtx");
const MAXP_TAG: u32 = u32::from_be_bytes(*b"maxp");
const NAME_TAG: u32 = u32::from_be_bytes(*b"name");
const _VHEA_TAG: u32 = u32::from_be_bytes(*b"vhea");

#[derive(Clone, Copy, Debug)]
pub enum TTFParseError {
    UnexpectedEOF,
    UnsupportedFormat,
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

        let unicode_table_offset = cmap_table
            .get_unicode_table()
            .ok_or(TTFParseError::MissingTable)?;
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
    pub fn get_glyph_id(&self, codepoint: u16) -> Option<GlyphID> {
        self.format4.get_glyph_id(codepoint)
    }

    pub fn get_glyph(&self, glyph_id: GlyphID) -> Result<Glyph<'a>, TTFParseError> {
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
        let glyphs = self.path_objects(text);
        for glyph in glyphs {
            // All the values are in font units, we need to scale them for appropriate use
            let scaled_x = self.scale(glyph.position.x, font_size) as usize;
            let scaled_y = self.scale(glyph.position.y, font_size) as usize;
            let scaled_width = self.scale(glyph.metrics.width(), font_size) as usize;
            let scaled_height = self.scale(glyph.metrics.height(), font_size) as usize;

            let mut rasterizer = Rasterizer::new(scaled_width, scaled_height);
            let scale_point = |glyph_point: DiscretePoint| Point {
                x: self.scale(glyph_point.x, font_size),
                y: self.scale(glyph_point.y, font_size),
            };

            // Draw the outlines of the glyph on the rasterizer buffer
            let mut write_head = Point::default();
            for path_op in glyph.path_operations {
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

            rasterizer.for_each_pixel(|pixel_position, opacity| {
                let translated_x = position.0 as usize + pixel_position.0 + scaled_x;
                let translated_y =
                    position.1 as usize + scaled_height - 1 - pixel_position.1 - scaled_y;

                let color = [255 - opacity; 3];

                // If the pixel is already darker, keep it as-is
                if 255 - opacity < bitmap.pixel_at(translated_x, translated_y)[0] {
                    bitmap
                        .pixel_at_mut(translated_x, translated_y)
                        .copy_from_slice(&color);
                }
            });
        }
    }

    fn path_objects<'b>(&'a self, text: &'b str) -> RenderedGlyphIterator<'a, 'b> {
        RenderedGlyphIterator::new(self, text)
    }

    /// Converts a value from font units to pixel size
    fn scale<V: Into<f32>>(&self, value: V, font_size: f32) -> f32 {
        (value.into() * font_size) / self.units_per_em() as f32
    }

    pub fn render_as_svg(&self, text: &str) -> String {
        let mut min_x = 0;
        let mut max_x = 0;
        let mut min_y = 0;
        let mut max_y = 0;

        let mut symbols = Vec::with_capacity(text.len());
        let mut symbol_positions = Vec::with_capacity(text.len());
        for (index, glyph) in self.path_objects(text).enumerate() {
            min_x = min_x.min(glyph.position.x + glyph.metrics.min_x);
            min_y = min_y.min(glyph.position.y + glyph.metrics.min_y);
            max_x = max_x.max(glyph.position.x + glyph.metrics.max_x);
            max_y = max_y.max(glyph.position.y + glyph.metrics.max_y);

            symbol_positions.push(glyph.position);

            let mut glyph_path = glyph
                .path_operations
                .map(|operation| match operation {
                    Operation::MoveTo(DiscretePoint { x, y }) => {
                        format!("M{x} {y}")
                    },
                    Operation::LineTo(DiscretePoint { x, y }) => {
                        format!("L{x} {y}")
                    },
                    Operation::QuadBezTo(p1, p2) => {
                        format!("Q{} {} {} {}", p1.x, p1.y, p2.x, p2.y)
                    },
                })
                .collect::<Vec<String>>()
                .join(" ");
            glyph_path.push_str(" Z");
            symbols.push(format!(
                "<symbol id=\"{index}\" overflow=\"visible\"><path d=\"{glyph_path}\"></path></symbol>"
            ));
        }

        let symbol_uses = symbol_positions
            .iter()
            .enumerate()
            .map(|(index, DiscretePoint { x, y })| {
                format!("<use xlink:href=\"#{index}\" x=\"{x}\" y=\"{y}\"/>")
            })
            .collect::<Vec<String>>()
            .join("");

        let width = max_x - min_x;
        let height = max_y - min_y;
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>
        <svg version=\"1.1\"
            xmlns=\"http://www.w3.org/2000/svg\"
            xmlns:xlink=\"http://www.w3.org/1999/xlink\"
            transform=\"scale(1, -1)\"
            viewBox=\"{min_x} {min_y} {width} {height}\">
          {} {}
        </svg>",
            symbols.join(""),
            symbol_uses,
        )
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

pub struct RenderedGlyph<'a> {
    metrics: Metrics,
    position: DiscretePoint,
    path_operations: PathReader<GlyphPointIterator<'a>>,
}

pub struct RenderedGlyphIterator<'a, 'b> {
    font: &'a Font<'a>,
    x: i16,
    y: i16,
    chars: std::str::Chars<'b>,
    current_compound_glyphs: Vec<CompoundGlyph<'a>>,
    advance_before_next_glyph: i16,
}

impl<'a, 'b> Iterator for RenderedGlyphIterator<'a, 'b> {
    type Item = RenderedGlyph<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        // Determine which glyph we should render and where we should render it to.
        // If we are currently in the process of emitting the components of some compound glyph, continue doing that
        // else, read the next character and emit that
        let (glyph_id, glyph_x, glyph_y) =
            if let Some(current_glyph) = self.current_compound_glyphs.last_mut() {
                if let Some(component) = current_glyph.next() {
                    (
                        component.glyph_id,
                        self.x + component.x_offset,
                        self.y + component.y_offset,
                    )
                } else {
                    // We are done emitting all parts of the current component glyph, pop it from the stack and start again
                    self.current_compound_glyphs.pop();
                    return self.next();
                }
            } else {
                let c = self.chars.next()?;
                self.x += self.advance_before_next_glyph;

                let glyph_id = self
                    .font
                    .get_glyph_id(c as u16)
                    .unwrap_or(GlyphID::REPLACEMENT);
                let horizontal_metrics = self.font.hmtx_table.get_metric_for(glyph_id);
                let glyph_x = self.x + horizontal_metrics.left_side_bearing();

                self.advance_before_next_glyph = horizontal_metrics.advance_width() as i16;

                (glyph_id, glyph_x, self.y)
            };

        let glyph = self.font.get_glyph(glyph_id).unwrap();

        match glyph {
            Glyph::Simple(simple_glyph) => {
                let path_operations = PathReader::new(simple_glyph.into_iter());
                Some(RenderedGlyph {
                    metrics: simple_glyph.metrics,
                    position: DiscretePoint {
                        x: glyph_x,
                        y: glyph_y,
                    },
                    path_operations,
                })
            },
            Glyph::Compound(compound_glyph) => {
                self.current_compound_glyphs.push(compound_glyph);
                self.next()
            },
        }
    }
}

impl<'a, 'b> RenderedGlyphIterator<'a, 'b> {
    pub fn new(font: &'a Font, text: &'b str) -> Self {
        Self {
            font: font,
            x: 0,
            y: 0,
            chars: text.chars(),
            current_compound_glyphs: vec![],
            advance_before_next_glyph: 0,
        }
    }
}
