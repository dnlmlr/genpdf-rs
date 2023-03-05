use crate::{
    math::{MathBlock, MathOp},
    render,
    style::LineStyle,
    Context, Element, Position, RenderResult,
};

/// An element that can render a MathBlock to a PDF document
pub struct MathElement {
    block: MathBlock,
}

impl MathElement {
    pub fn new(block: MathBlock) -> Self {
        Self { block }
    }
}

impl Element for MathElement {
    fn render(
        &mut self,
        context: &Context,
        area: render::Area<'_>,
        style: crate::style::Style,
    ) -> Result<RenderResult, crate::error::Error> {
        for op in self.block.ops() {
            match op {
                MathOp::TextSection(text_section) => {
                    area.print_positioned_codepoints(
                        &context.font_cache,
                        Position::new(0, text_section.origin_y),
                        text_section.x_offsets.clone().into_iter(),
                        text_section.glyph_ids.clone().into_iter(),
                        text_section.font_size,
                        style.with_color(text_section.color),
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

        Ok(RenderResult::default())
    }
}
