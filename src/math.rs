use rex::Backend;

pub enum MathOp {
    Glyph {
        x: f64,
        y: f64,
        scale: f64,
        gid: u16,
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
}

const PX_TO_MM: f64 = 0.352777;

impl Backend for MathBlock {
    fn symbol(&mut self, pos: rex::Cursor, gid: u16, scale: f64, ctx: &rex::MathFont) {
        // todo batch text areas of same y offset and scale together
        self.math_ops.push(MathOp::Glyph {
            x: pos.x as f64 * PX_TO_MM, // value is in EM
            y: pos.y as f64 * PX_TO_MM,
            gid,
            scale, // scale = actual font_size for character
        });
    }

    fn rule(&mut self, pos: rex::Cursor, width: f64, height: f64) {
        self.math_ops.push(MathOp::Rect {
            x: pos.x * PX_TO_MM,
            y: (pos.y + 9.2) * PX_TO_MM,
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
