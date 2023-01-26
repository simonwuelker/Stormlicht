use canvas::{Canvas, Drawable};
use std::collections::HashMap;

use crate::{
    bezier::QuadraticBezier,
    ttf::{self, tables::glyf::GlyphPoint},
};

const DEFAULT_FONT: &[u8; 168644] =
    include_bytes!("../../downloads/fonts/roboto/Roboto-Medium.ttf");

#[derive(Clone, Copy, Debug, Default)]
pub struct BoundingBox {
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct LayoutInfo {
    bounding_box: BoundingBox,
    advance_width: usize,
    advance_height: usize,
}

#[derive(Debug, Default)]
pub struct Glyph {
    curves: Vec<QuadraticBezier>,
    layout_info: LayoutInfo,
}

pub struct Font {
    name: Option<String>,
    units_per_em: f32,
    glyphs: Vec<Glyph>,
    glyph_indices: HashMap<u16, u16>,
}

impl Default for Font {
    fn default() -> Self {
        Self::from_bytes(DEFAULT_FONT)
    }
}

impl Font {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let ttf_font = ttf::Font::new(bytes).unwrap();
        let name = ttf_font.name().get_font_name();
        let units_per_em = ttf_font.units_per_em() as f32;

        let num_glyphs = ttf_font.num_glyphs();
        let mut glyphs = Vec::with_capacity(num_glyphs);
        let mut glyph_indices = HashMap::with_capacity(num_glyphs);

        let format_4 = ttf_font.format_4();
        format_4.codepoints(|codepoint| {
            // Add the "character -> glyph index" mapping
            let glyph_index = format_4.get_glyph_index(codepoint).unwrap_or(1);
            glyph_indices.insert(codepoint, glyph_index);

            // Add the actual glyph
            let mut ttf_glyph = ttf_font.glyf().get_glyph(glyph_index);

            if !ttf_glyph.is_simple() {
                // TODO: We can't render complex glyphs yet, use the missing glyph instead
                ttf_glyph = ttf_font.glyf().get_glyph(1);
            }

            let glyph_outline = ttf_glyph.outline();
            let mut glyph = Glyph::from_glyph_points(glyph_outline.points());

            // Compute glyph layout stuff
            glyph.layout_info.advance_width =
                ttf_font.hmtx().get_metric_for(glyph_index).advance_width();
            glyph.layout_info.bounding_box = BoundingBox {
                min_x: ttf_glyph.min_x() as f32 / units_per_em,
                min_y: ttf_glyph.min_y() as f32 / units_per_em,
                max_x: ttf_glyph.max_x() as f32 / units_per_em,
                max_y: ttf_glyph.max_y() as f32 / units_per_em,
            };

            glyphs.push(glyph);
        });

        Self {
            name: name,
            units_per_em: units_per_em,
            glyphs: glyphs,
            glyph_indices: glyph_indices,
        }
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
        let scale = font_size / self.units_per_em;
        let mut x = 0;
        for c in text.chars() {
            let glyph = self.get_glyph(c as u16);

            let bounding_box = &glyph.layout_info.bounding_box;
            let max_x = ((bounding_box.max_x + bounding_box.min_x) * font_size).ceil() as usize;
            let min_y = (bounding_box.min_y * font_size).ceil() as usize;
            let max_y = (bounding_box.max_y * font_size).ceil() as usize;

            let mut render_to = canvas.borrow(x..x + max_x, min_y..=max_y);

            for mut curve in glyph.curves().iter().copied() {
                curve.scale(scale);
                render_to.quad_bezier(
                    (
                        curve.p0.x.round() as usize,
                        render_to.height() - curve.p0.y.round() as usize,
                    ),
                    (
                        curve.p1.x.round() as usize,
                        render_to.height() - curve.p1.y.round() as usize,
                    ),
                    (
                        curve.p2.x.round() as usize,
                        render_to.height() - curve.p2.y.round() as usize,
                    ),
                    color,
                );
            }

            x += (font_size * glyph.advance_width() as f32 / self.units_per_em).ceil() as usize;
        }
    }

    /// Get a glyph by it's index.
    /// If possible, you should prefer to use [Self::get_glyph] instead.
    fn get_indexed(&self, index: usize) -> &Glyph {
        &self.glyphs[index]
    }
}

impl Glyph {
    pub fn advance_width(&self) -> usize {
        self.layout_info.advance_width
    }

    pub fn advance_height(&self) -> usize {
        self.layout_info.advance_height
    }

    pub fn curves(&self) -> &[QuadraticBezier] {
        &self.curves
    }

    pub fn from_glyph_points<I: Iterator<Item = GlyphPoint>>(points: I) -> Self {
        let mut glyph = Self::default();
        let mut previous_point: Option<GlyphPoint> = None;
        let mut first_point_of_contour = None;

        // TODO these aren't actually bezier curves!
        for point in points {
            match previous_point {
                Some(previous_point) => {
                    glyph.curves.push(QuadraticBezier {
                        p0: previous_point.into(),
                        p1: previous_point.into(),
                        p2: point.into(),
                    });
                },
                None => first_point_of_contour = Some(point),
            }

            // It is technically possible, while pointless (hah) to have
            // a contour containing only a single point - in which case
            // point.is_last_point_of_contour is true but first_point_of_contour
            // is None. In this case, we silently ignore the point and move on.
            if let (Some(first_point), true) =
                (first_point_of_contour, point.is_last_point_of_contour)
            {
                glyph.curves.push(QuadraticBezier {
                    p0: point.into(),
                    p1: point.into(),
                    p2: first_point.into(),
                });
                previous_point = None;
            } else {
                previous_point = Some(point);
            }
        }
        glyph
    }
}
