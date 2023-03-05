use crate::{math::MathBlock, render, style::LineStyle, Context, Element, Position, RenderResult};

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
                crate::math::MathOp::Rect {
                    x,
                    y,
                    width,
                    height,
                } => area.draw_line(
                    vec![
                        Position::new(*x, *y),
                        Position::new(x + width, *y),
                        Position::new(x + width, y + height),
                        Position::new(*x, y + height),
                        Position::new(*x, *y),
                    ],
                    LineStyle::default().with_filled(true),
                ),
                crate::math::MathOp::TextSection {
                    origin_y,
                    font_size,
                    x_offsets,
                    glyph_ids,
                    ..
                } => area.print_positioned_codepoints(
                    &context.font_cache,
                    Position::new(0, *origin_y),
                    x_offsets.into_iter().map(|f| *f),
                    glyph_ids.into_iter().map(|f| *f),
                    *font_size,
                ),
            }
        }

        Ok(RenderResult::default())
    }
}
