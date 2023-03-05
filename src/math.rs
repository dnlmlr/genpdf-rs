use font::Font;
use rex::Backend;

const POSITIONING_ACCURACY: f64 = 0.01; // Unit: millimeters

fn mm_to_em(mm: f64, font_size: f64) -> f64 {
    mm * (1.0 / font_size) * 2.835 // todo what is this magic number
}

pub enum MathOp {
    TextSection {
        origin_y: f64,
        font_size: f64,
        x_offsets: Vec<f64>,
        glyph_ids: Vec<u16>,
        last_x: f64,
    },
    Rect {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    },
}

pub struct MathBlock {
    math_ops: Vec<MathOp>,
}

impl MathBlock {
    pub fn new() -> Self {
        Self {
            math_ops: Vec::new(),
        }
    }

    pub fn ops(&self) -> &[MathOp] {
        &self.math_ops
    }

    fn push_glyph(&mut self, x: f64, y: f64, glyph_font_size: f64, glyph_id: u16, adv: f64) {
        for op in &mut self.math_ops {
            match op {
                MathOp::TextSection {
                    origin_y,
                    font_size,
                    x_offsets,
                    glyph_ids,
                    last_x,
                } if (y - *origin_y).abs() < POSITIONING_ACCURACY
                    && (glyph_font_size - *font_size).abs() < f64::EPSILON =>
                {
                    let x_pos = mm_to_em(x, glyph_font_size);

                    x_offsets.push(x_pos - *last_x);
                    glyph_ids.push(glyph_id);

                    *last_x = x_pos + adv;

                    return;
                }
                _ => {}
            }
        }

        self.math_ops.push(MathOp::TextSection {
            origin_y: y,
            font_size: glyph_font_size,
            x_offsets: vec![mm_to_em(x, glyph_font_size)],
            glyph_ids: vec![glyph_id],
            last_x: mm_to_em(x, glyph_font_size) + adv,
        });
    }
}

const PX_TO_MM: f64 = 0.352777;

impl Backend for MathBlock {
    fn symbol(&mut self, pos: rex::Cursor, gid: u16, scale: f64, ctx: &rex::MathFont) {
        let scale_vec = ctx.font_matrix().extract_scale();

        let adv = (ctx.glyph_metrics(gid).unwrap().advance * scale_vec.x()) as f64;
        self.push_glyph(
            pos.x as f64 * PX_TO_MM,
            pos.y as f64 * PX_TO_MM,
            scale,
            gid,
            adv as f64,
        );
    }

    fn rule(&mut self, pos: rex::Cursor, width: f64, height: f64) {
        self.math_ops.push(MathOp::Rect {
            x: pos.x * PX_TO_MM,
            y: (pos.y + 9.15) * PX_TO_MM,
            height: height * PX_TO_MM,
            width: width * PX_TO_MM,
        });
    }

    fn begin_color(&mut self, color: rex::RGBA) {
        todo!()
    }

    fn end_color(&mut self) {
        todo!()
    }
}
