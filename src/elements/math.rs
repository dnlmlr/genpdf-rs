use crate::{
    math::MathOp, render, style::LineStyle, Context, Element, Position, RenderResult, Size,
};

/// An element that can render a MathBlock to a PDF document
pub struct Math {
    source: String,
}

impl Math {
    /// Creates a new math element that renders the given LaTeX math source
    pub fn new(source: &str) -> Self {
        Self {
            source: source.to_owned(),
        }
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

        for op in block.ops() {
            match op {
                MathOp::TextSection(text_section) => {
                    area.print_positioned_codepoints(
                        &context.font_cache,
                        Position::new(0, text_section.origin_y),
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
                        Position::new(rule.x, rule.y),
                        Position::new(rule.x + rule.width, rule.y),
                        Position::new(rule.x + rule.width, rule.y + rule.height),
                        Position::new(rule.x, rule.y + rule.height),
                        Position::new(rule.x, rule.y),
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
