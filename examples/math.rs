use font::OpenTypeFont;
use genpdf::{
    elements::{self, MathElement},
    fonts::FontFamily,
    math::MathBlock,
};
use rex::{
    font::FontContext,
    layout::{engine, Grid, Layout, LayoutSettings},
    parser, Renderer,
};

const MATH_EXAMPLE: &'static str = r"
\mathop{\overbrace{1+2+3+\unicodecdots+n}}\limits^{\mathrm{Arithmatic}} = \frac{n(n+1)}{2}  \mid
\frac{1}{\left(\sqrt{\phi\sqrt5} - \phi\right) e^{\frac{2}{5}\pi}} = 1 + \frac{e^{-2\pi}}{1 + \frac{e^{-4\pi}}{1 + \frac{e^{-6\pi}}{1 + \frac{e^{-8\pi}}{1 + \unicodecdots}}}}  \mid
\left\vert\sum_k a_k b_k\right\vert \leq \left(\sum_k a_k^2\right)^{\frac12}\left(\sum_k b_k^2\right)^{\frac12}
\mathop{\mathrm{lim\,sup}}\limits_{x\rightarrow\infty}\ \mathop{\mathrm{sin}}(x)\mathrel{\mathop{=}\limits^?}1
f^{(n)}(z) = \frac{n!}{2\pi i} \oint \frac{f(\xi)}{(\xi - z)^{n+1}}\,\mathrm{d}\xi
";

fn main() {
    let xits_font = include_bytes!("./fonts/rex-xits.ttf");

    let pdf_font = genpdf::fonts::FontData::new(xits_font.to_vec(), None).unwrap();
    let pdf_font_family = FontFamily {
        regular: pdf_font.clone(),
        bold: pdf_font.clone(),
        italic: pdf_font.clone(),
        bold_italic: pdf_font,
    };

    let mut pdf_doc = genpdf::Document::new(pdf_font_family);
    pdf_doc.set_title("genpdf+rex Demo Document");
    pdf_doc.set_minimal_conformance();
    pdf_doc.set_line_spacing(1.25);
    pdf_doc.set_font_size(10);

    let rex_font = font::parse(xits_font)
        .unwrap()
        .downcast_box::<OpenTypeFont>();

    if let Ok(rex_font) = rex_font {
        assert!(rex_font.math.is_some());

        let rex_font_ctx = FontContext::new(&rex_font);
        let rex_layout_settings = LayoutSettings::new(
            &rex_font_ctx,
            pdf_doc.font_size() as f64,
            rex::layout::Style::Display,
        );
        let rex_ast = parser::parse(MATH_EXAMPLE).unwrap();
        let rex_math_block = engine::layout(&rex_ast, rex_layout_settings).unwrap();

        let mut math_block = MathBlock::new();

        let rex_renderer = Renderer::new();
        let mut rex_grid = Grid::new();
        rex_grid.insert(0, 0, rex_math_block.as_node());

        let mut rex_layout = Layout::new();
        rex_layout.add_node(rex_grid.build());
        rex_renderer.render(&rex_layout, &mut math_block);

        pdf_doc.push(elements::Text::new("Math with Rex and genpdf"));
        pdf_doc.push(MathElement::new(math_block));

        pdf_doc
            .render_to_file("./math.pdf")
            .expect("Failed to write output file");
    }
}
