//! Implements a ReX-based math renderer for genpdf

use std::fmt::Debug;

use font::{Font, OpenTypeFont};
use rex::{font::FontContext, parser::ParseNode, Backend};

use crate::{fonts::FontFamily, style::Color, Size};

/// Maximum difference between two y-offsets to be inserted into same batch
const POSITIONING_ACCURACY: f64 = 0.01; // Unit: millimeters

/// ReX renders at 72dpi; 1in = 25.4mm; 25.4mm / 72 dpi = 0.352777
// const PX_TO_MM: f64 = 0.3514598035146;
const PX_TO_MM: f64 = 25.4 / 72.0;

/// Converts millimeters to EMs at the given font size. Useful for calculating kerning.
fn mm_to_em(mm: f64, font_size: f64) -> f64 {
    let pixels = mm / PX_TO_MM;
    pixels * (1.0 / font_size)
}

/// A batch of glyphs with the same color, y-offset and size, that can be rendered as a single text seciton
pub struct TextSection {
    /// Vertical offset of the text section, in mm
    pub y_origin: f64,

    /// Font size of the text section, in em
    pub font_size: f64,

    /// Horizontal offsets of the glyphs, in em
    pub x_offsets: Vec<f64>,

    /// IDs of the glyphs in the section
    pub glyph_ids: Vec<u16>,

    /// Color of the section
    pub color: Color,

    last_x: f64,
}

impl TextSection {
    fn can_append(&self, y_origin: f64, font_size: f64, color: Color) -> bool {
        (self.y_origin - y_origin).abs() < POSITIONING_ACCURACY
            && (self.font_size - font_size) < f64::EPSILON
            && self.color == color
    }
}

/// A colored, filled rectangle
pub struct Rule {
    /// Rect x origin in mm
    pub x: f64,
    /// Rect y origin in mm
    pub y: f64,
    /// Rect Width in mm
    pub width: f64,
    /// Rect Height in mm
    pub height: f64,
    /// Rect Color
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
    /// Bounding box of the math block
    pub size: Size,
    math_ops: Vec<MathOp>,
    current_color: Color,
}

impl MathBlock {
    /// Creates a new, empty math block with black color and the given bounding box
    pub fn new(size: Size) -> Self {
        Self {
            size,
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
                    y_origin: y,
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
        let font_scale_x = ctx.font_matrix().extract_scale().x();
        let font_scale_y = ctx.font_matrix().extract_scale().y();

        // rex positions everything one 1em too low if the grid is used.
        // but if the grid is NOT used, the y offsets are completely wrong, so just fix it here
        let ascend = ctx.vmetrics().map(|it| it.ascent).unwrap_or(0.0);
        let y_offset = -(ascend * font_scale_y) as f64 * font_size;
        let advance = ctx
            .glyph_metrics(gid)
            .map(|metrics| metrics.advance)
            .unwrap_or(0.0)
            * font_scale_x;

        self.push_glyph(
            pos.x as f64 * PX_TO_MM,
            (pos.y as f64 + y_offset) * PX_TO_MM,
            gid,
            font_size,
            self.current_color,
            advance as f64,
        );
    }

    fn rule(&mut self, pos: rex::Cursor, width: f64, height: f64) {
        self.math_ops.push(MathOp::Rule(Rule {
            x: pos.x * PX_TO_MM,
            y: pos.y * PX_TO_MM,
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

/// Wrapper for the ReX renderer structure
pub struct MathRenderer {
    font_family: FontFamily<crate::fonts::Font>,
    font: Box<OpenTypeFont>,
    rex_renderer: rex::Renderer,
}

impl Debug for MathRenderer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // todo
        f.debug_struct("MathRenderer").finish()
    }
}

impl MathRenderer {
    pub(crate) fn new(
        math_font_data: &[u8],
        math_font_family: FontFamily<crate::fonts::Font>,
    ) -> Self {
        let font = font::parse(math_font_data)
            .expect("Failed to decode math font")
            .downcast_box::<OpenTypeFont>();

        match font {
            Ok(font) => Self {
                font,
                rex_renderer: rex::Renderer::new(),
                font_family: math_font_family,
            },
            Err(_) => panic!("Not an OpenType font"),
        }
    }

    pub(crate) fn render(&self, font_size: f64, rex_ast: &[ParseNode]) -> MathBlock {
        use rex::{
            layout::engine::layout,
            layout::{Grid, Layout, LayoutSettings},
        };

        let rex_font_ctx = FontContext::new(&self.font); // todo maybe don't reinstantiate every time
        let rex_layout_settings =
            LayoutSettings::new(&rex_font_ctx, font_size, rex::layout::Style::Display);

        // Todo: Figure out if this can reasonably panic or not
        let rex_math_block = layout(&rex_ast, rex_layout_settings).expect("Failed to layout math");

        let mut rex_grid = Grid::new();
        rex_grid.insert(0, 0, rex_math_block.as_node());

        let mut rex_layout = Layout::new();
        rex_layout.add_node(rex_grid.build());

        let (x0, y0, x1, y1) = self.rex_renderer.size(&rex_layout);
        let size = Size::new((x1 - x0) * PX_TO_MM, (y1 - y0) * PX_TO_MM);
        let mut math_block = MathBlock::new(size);
        self.rex_renderer.render(&rex_layout, &mut math_block);

        math_block
    }

    pub(crate) fn font_family(&self) -> FontFamily<crate::fonts::Font> {
        self.font_family
    }
}
