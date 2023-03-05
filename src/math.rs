use font::Font;
use rex::Backend;

use crate::style::Color;

/// Rex applies a vertical offset to all lines... TODO: we need to find out what this actually means
const REX_V_OFFSET: f64 = 7.6;

/// Maximum difference between two y-offsets to be inserted into same batch
const POSITIONING_ACCURACY: f64 = 0.01; // Unit: millimeters

/// PDFs have 72dpi; 1in = 25.4mm; 25.4mm / 72 dpi = 0.352777
const PX_TO_MM: f64 = 0.352777;

/// Converts millimeters to EMs at the given font size. Useful for calculating kerning.
fn mm_to_em(mm: f64, font_size: f64) -> f64 {
    let pixels = mm / PX_TO_MM;
    pixels * (1.0 / font_size)
}

/// A batch of glyphs with the same color, y-offset and size, that can be rendered as a single text seciton
pub struct TextSection {
    pub origin_y: f64,
    pub font_size: f64,
    pub x_offsets: Vec<f64>,
    pub glyph_ids: Vec<u16>,
    pub color: Color,

    last_x: f64,
}

impl TextSection {
    fn can_append(&self, origin_y: f64, font_size: f64, color: Color) -> bool {
        (self.origin_y - origin_y).abs() < POSITIONING_ACCURACY
            && (self.font_size - font_size) < f64::EPSILON
            && self.color == color
    }
}

/// A colored, filled rectangle
pub struct Rule {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub color: Color,
}

/// A drawing command
pub enum MathOp {
    /// See TextSection for details
    TextSection(TextSection),

    /// See Rule for details.
    Rule(Rule),
}

/// Receives and holds math drawing commands from ReX
pub struct MathBlock {
    math_ops: Vec<MathOp>,
    current_color: Color,
}

impl MathBlock {
    /// Creates a new, empty math block with black color
    pub fn new() -> Self {
        Self {
            math_ops: Vec::new(),
            current_color: Color::Rgb(0, 0, 0),
        }
    }

    /// Gets the list of drawing operations required by this block
    pub fn ops(&self) -> &[MathOp] {
        &self.math_ops
    }

    fn find_matching_text_section(
        &mut self,
        y: f64,
        font_size: f64,
        color: Color,
    ) -> Option<&mut TextSection> {
        for op in &mut self.math_ops {
            match op {
                MathOp::TextSection(section) if section.can_append(y, font_size, color) => {
                    return Some(section)
                }
                _ => {}
            }
        }

        None
    }

    fn push_glyph(
        &mut self,
        x: f64,
        y: f64,
        glyph_id: u16,
        font_size: f64,
        color: Color,
        advance: f64,
    ) {
        let section = self.find_matching_text_section(y, font_size, color);

        match section {
            Some(section) => {
                let x_pos = mm_to_em(x, font_size);
                section.x_offsets.push(x_pos - section.last_x);
                section.glyph_ids.push(glyph_id);
                section.last_x = x_pos + advance;
            }
            None => {
                self.math_ops.push(MathOp::TextSection(TextSection {
                    origin_y: y,
                    font_size,
                    x_offsets: vec![mm_to_em(x, font_size)],
                    glyph_ids: vec![glyph_id],
                    last_x: mm_to_em(x, font_size) + advance,
                    color,
                }));
            }
        }
    }
}

impl Backend for MathBlock {
    fn symbol(&mut self, pos: rex::Cursor, gid: u16, font_size: f64, ctx: &rex::MathFont) {
        let font_scale = ctx.font_matrix().extract_scale().x();
        let advance = ctx
            .glyph_metrics(gid)
            .map(|metrics| metrics.advance)
            .unwrap_or(0.0)
            * font_scale;

        self.push_glyph(
            pos.x as f64 * PX_TO_MM,
            pos.y as f64 * PX_TO_MM,
            gid,
            font_size,
            self.current_color,
            advance as f64,
        );
    }

    fn rule(&mut self, pos: rex::Cursor, width: f64, height: f64) {
        self.math_ops.push(MathOp::Rule(Rule {
            x: pos.x * PX_TO_MM,
            y: (pos.y + REX_V_OFFSET) * PX_TO_MM,
            height: height * PX_TO_MM,
            width: width * PX_TO_MM,
            color: self.current_color,
        }));
    }

    fn begin_color(&mut self, color: rex::RGBA) {
        self.current_color = Color::Rgb(color.0, color.1, color.2);
    }

    fn end_color(&mut self) {
        self.current_color = Color::Rgb(0, 0, 0);
    }
}
