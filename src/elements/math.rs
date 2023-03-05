use crate::{
    math::MathOp, render, style::LineStyle, Alignment, Context, Element, Position, RenderResult,
};

/// An element that can render LaTeX-styled math formulas to a PDF document
pub struct Math {
    source: String,
    alignment: Alignment,
}

impl Math {
    /// Creates a new math element that renders the given LaTeX math source
    pub fn new(source: &str) -> Self {
        Self {
            source: source.to_owned(),
            alignment: Alignment::Left,
        }
    }

    /// Sets the horizontal alignment of the Math block
    pub fn aligned(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }
}

impl Element for Math {
    fn render(
        &mut self,
        context: &Context,
        area: render::Area<'_>,
        style: crate::style::Style,
    ) -> Result<RenderResult, crate::error::Error> {
        let math_renderer = context
            .math_renderer
            .as_ref()
            .expect("Tried to use math element without an active math font");

        let block = math_renderer.render(style.font_size() as f64, &self.source);

        let x_origin = match self.alignment {
            Alignment::Left | Alignment::Justified(_) => 0.0,
            Alignment::Center => (area.size().width / 2.0 - block.size.width / 2.0).0,
            Alignment::Right => (area.size().width - block.size.width).0,
        };

        for op in block.ops() {
            match op {
                MathOp::TextSection(text_section) => {
                    area.print_positioned_codepoints(
                        &context.font_cache,
                        Position::new(x_origin, text_section.origin_y),
                        text_section.x_offsets.clone().into_iter(),
                        text_section.glyph_ids.clone().into_iter(),
                        text_section.font_size,
                        style
                            .with_color(text_section.color)
                            .with_font_family(math_renderer.font_family()),
                    );
                }
                MathOp::Rule(rule) => area.draw_line(
                    vec![
                        Position::new(x_origin + rule.x, rule.y),
                        Position::new(x_origin + rule.x + rule.width, rule.y),
                        Position::new(x_origin + rule.x + rule.width, rule.y + rule.height),
                        Position::new(x_origin + rule.x, rule.y + rule.height),
                        Position::new(x_origin + rule.x, rule.y),
                    ],
                    LineStyle::default()
                        .with_color(rule.color)
                        .with_filled(true),
                ),
            }
        }

        let mut result = RenderResult::default();
        result.size = block.size;
        Ok(result)
    }
}
