use anyhow::Result;
use canvas::{Canvas, Drawable};
use std::collections::HashMap;

use crate::{
    bezier::{Line, Point},
    ttf,
};

const DEFAULT_FONT: &[u8; 168644] =
    include_bytes!("../../downloads/fonts/roboto/Roboto-Medium.ttf");

#[derive(Clone, Copy, Debug, Default)]
/// A rectangular bounding box, units are in `em`.
pub struct BoundingBox {
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct LayoutInfo {
    bounding_box: BoundingBox,
    /// The amount of space the reader should advance by horizontally after
    /// rendering this glyph, in `em`
    advance_width: f32,
    advance_height: f32,
    left_side_bearing: f32,
}

#[derive(Debug, Default, Clone)]
pub struct Glyph {
    curves: Vec<Line>,
    layout_info: LayoutInfo,
}

pub struct Font {
    name: Option<String>,
    max_bb: BoundingBox,
    units_per_em: f32,
    glyphs: Vec<Glyph>,
    glyph_indices: HashMap<u16, u16>,
}

impl Default for Font {
    fn default() -> Self {
        Self::from_bytes(DEFAULT_FONT).unwrap()
    }
}

impl Font {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let ttf_font = ttf::Font::new(bytes)?;
        let name = ttf_font.name().get_font_name();
        let units_per_em = ttf_font.units_per_em() as f32;

        let num_glyphs = ttf_font.num_glyphs();
        let mut glyphs = vec![Glyph::default(); num_glyphs];
        let mut glyph_indices = HashMap::with_capacity(num_glyphs);

        let head = ttf_font.head();
        let max_bb = BoundingBox {
            min_x: head.min_x() as f32 / units_per_em,
            min_y: head.min_y() as f32 / units_per_em,
            max_x: head.max_x() as f32 / units_per_em,
            max_y: head.max_y() as f32 / units_per_em,
        };

        let format_4 = ttf_font.format_4();
        format_4.codepoints(|codepoint| {
            // Add the "character -> glyph index" mapping
            let glyph_index = format_4.get_glyph_index(codepoint).unwrap_or(1);
            glyph_indices.insert(codepoint, glyph_index);

            // Add the actual glyph
            let mut glyph = &mut glyphs[glyph_index as usize];
            let ttf_glyph = ttf_font.glyf().get_glyph(glyph_index);
            ttf_glyph
                .compute_outline(ttf_font.glyf(), &mut glyph.curves)
                .unwrap();

            // Compute glyph layout stuff
            let horizontal_metric = ttf_font.hmtx().get_metric_for(glyph_index);
            glyph.layout_info.advance_width =
                horizontal_metric.advance_width() as f32 / units_per_em;
            glyph.layout_info.left_side_bearing =
                horizontal_metric.left_side_bearing() as f32 / units_per_em;

            glyph.layout_info.bounding_box = BoundingBox {
                min_x: ttf_glyph.min_x() as f32 / units_per_em,
                min_y: ttf_glyph.min_y() as f32 / units_per_em,
                max_x: ttf_glyph.max_x() as f32 / units_per_em,
                max_y: ttf_glyph.max_y() as f32 / units_per_em,
            };
        });

        Ok(Self {
            name: name,
            units_per_em: units_per_em,
            max_bb: max_bb,
            glyphs: glyphs,
            glyph_indices: glyph_indices,
        })
    }

    /// Return the number of coordinate points per font size unit.
    /// This value is used to scale fonts, ie. when you render a font with
    /// size `17px`, one `em` equals `17px`.
    ///
    /// Note that this value does not constrain the size of individual glyphs.
    /// A glyph may have a size larger than `1em`.
    pub fn units_per_em(&self) -> f32 {
        self.units_per_em
    }

    /// Get the number of glyphs defined by the font
    pub fn num_glyphs(&self) -> usize {
        self.glyphs.len()
    }

    /// Get the full name of the font, if specified.
    /// Fonts will usually specify their own name, though it is not required.
    pub fn name(&self) -> Option<&String> {
        self.name.as_ref()
    }

    /// Find the glyph for a given character.
    /// If the font does not contain the character,
    /// the replacement glyph (index 0) is returned instead.
    pub fn get_glyph(&self, codepoint: u16) -> &Glyph {
        self.get_indexed(self.get_glyph_index(codepoint))
    }

    fn get_glyph_index(&self, codepoint: u16) -> usize {
        self.glyph_indices
            .get(&codepoint)
            .copied()
            .unwrap_or_default() as usize
    }

    pub fn rasterize(&self, text: &str, canvas: &mut Canvas, font_size: f32, color: &[u8]) {
        let mut glyph_bb = self.max_bb;
        glyph_bb.scale(font_size);

        let scale = font_size / self.units_per_em;
        let mut x = 0;
        for c in text.chars() {
            let glyph = self.get_glyph(c as u16);
            dbg!(x, font_size);
            let mut render_to = canvas.borrow(
                x..x + glyph_bb.width().ceil() as usize,
                0..glyph_bb.height().ceil() as usize,
            );
            render_to.fill(&[0, 0, 255]);

            let min_y = glyph.layout_info.bounding_box.min_y * font_size;
            let max_y = glyph.layout_info.bounding_box.max_y * font_size;
            let map_point =
                |p: Point| (p.x.round() as usize, (max_y - p.y - min_y).round() as usize);

            for mut curve in glyph.curves().iter().copied() {
                curve.scale(scale);
                match curve {
                    Line::Straight(p0, p1) => render_to.line(map_point(p0), map_point(p1), color),
                    Line::Quad(bez) => render_to.quad_bezier(
                        map_point(bez.p0),
                        map_point(bez.p1),
                        map_point(bez.p2),
                        color,
                    ),
                }
            }

            x += (font_size * glyph.advance_width()).ceil() as usize;
        }
    }

    /// Get a glyph by it's index.
    /// If possible, you should prefer to use [Self::get_glyph] instead.
    fn get_indexed(&self, index: usize) -> &Glyph {
        &self.glyphs[index]
    }
}

impl Glyph {
    pub fn advance_width(&self) -> f32 {
        self.layout_info.advance_width
    }

    pub fn advance_height(&self) -> f32 {
        self.layout_info.advance_height
    }

    pub fn curves(&self) -> &[Line] {
        &self.curves
    }
}

impl LayoutInfo {
    pub fn width(&self, font_size: f32) -> usize {
        (self.advance_width * font_size).ceil() as usize
        // ((self.left_side_bearing + self.bounding_box.max_x - self.bounding_box.min_x) * font_size)
        //     .ceil() as usize
    }

    pub fn height(&self, font_size: f32) -> usize {
        ((self.bounding_box.max_y - self.bounding_box.min_y) * font_size).ceil() as usize
    }
}

impl BoundingBox {
    pub fn scale(&mut self, scale: f32) {
        self.min_x *= scale;
        self.max_x *= scale;
        self.min_y *= scale;
        self.max_y *= scale;
    }

    /// Width of the bounding box, in `em`
    pub fn width(&self) -> f32 {
        self.max_x - self.min_x
    }

    /// Height of the bounding box, in `em`
    pub fn height(&self) -> f32 {
        self.max_y - self.min_y
    }
}
